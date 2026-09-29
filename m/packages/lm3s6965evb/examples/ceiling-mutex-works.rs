//! Priority ceiling mutex works
//@ runner: $RUNNER
//@ target: $TARGET

#![no_std]
#![no_main]

use core::sync::atomic::{self, AtomicU16, AtomicUsize};

use m_rt::interrupt_handler;
use m_rt::nvic::{HandlerState, Mutex, MutexRef, Nvic};
use m_rt::vtor::ExternalInterrupt;
use sh::eprintln;
use statically::owned;

m_rt::entry!(main);

fn main() -> ! {
    let nvic = Nvic::acquire();

    let mut interrupts = nvic.interrupts();
    let msg = "this test needs at least three implemented interrupts";

    let max_prio = nvic.max_group_priority();
    eprintln!("max_prio={max_prio}");

    let mutex = owned!(Mutex<()>, Mutex::new(()));
    let mutex = nvic.set_ceiling(1, mutex);

    {
        let first = interrupts.next().expect(msg);
        let first_handler = interrupt_handler!(FirstHandler);
        nvic.set_stateful_handler(first, first_handler, FirstHandler { mutex });
        nvic.set_group_priority(first, 0);
        Nvic::enable(first);
        Nvic::set_pending(first);
    }

    {
        let second = interrupts.next().expect(msg);
        let second_handler = interrupt_handler!(SecondHandler);
        nvic.set_stateful_handler(second, second_handler, SecondHandler { mutex });
        nvic.set_group_priority(second, 1);
        Nvic::enable(second);
        SECOND.store(second.into(), atomic::Ordering::Relaxed);
    }

    {
        let third = interrupts.next().expect(msg);
        nvic.set_handler(third, third_handler);
        nvic.set_group_priority(third, 2);
        Nvic::enable(third);
        THIRD.store(third.into(), atomic::Ordering::Relaxed);
    }

    nvic.enable_interrupts();
    assert_eq!(6, count());

    sh::exit()
}

static SECOND: AtomicU16 = AtomicU16::new(0);
static THIRD: AtomicU16 = AtomicU16::new(0);

struct FirstHandler {
    mutex: MutexRef<()>,
}

impl HandlerState for FirstHandler {
    fn on_interrupt(&mut self) {
        assert_eq!(0, count());
        self.mutex.lock(|_| {
            let second = ExternalInterrupt(SECOND.load(atomic::Ordering::Relaxed));
            Nvic::set_pending(second);
            assert_eq!(1, count());
        });
        assert_eq!(5, count());
    }
}

struct SecondHandler {
    mutex: MutexRef<()>,
}

impl HandlerState for SecondHandler {
    fn on_interrupt(&mut self) {
        assert_eq!(2, count());
        self.mutex.lock(|_| {
            let third = ExternalInterrupt(THIRD.load(atomic::Ordering::Relaxed));
            // this exception is allowed to preempt this critical section
            Nvic::set_pending(third);
            assert_eq!(4, count());
        });
    }
}

extern "C" fn third_handler() {
    assert_eq!(3, count());
}

fn count() -> usize {
    static COUNT: AtomicUsize = AtomicUsize::new(0);

    COUNT.fetch_add(1, atomic::Ordering::Relaxed)
}

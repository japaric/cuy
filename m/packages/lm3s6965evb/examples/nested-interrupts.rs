//! interrupt nesting works with smallest and largest group priorities
//@ runner: $RUNNER
//@ target: $TARGET

#![no_std]
#![no_main]

use core::sync::atomic::{self, AtomicU16, AtomicUsize};

use m_rt::nvic::Nvic;
use m_rt::vtor::ExternalInterrupt;
use sh::eprintln;

m_rt::entry!(main);

fn main() -> ! {
    let nvic = Nvic::acquire();

    let mut interrupts = nvic.interrupts();
    let msg = "this test needs at least four implemented interrupts";
    let first = interrupts.next().expect(msg);
    let second = interrupts.next().expect(msg);
    let third = interrupts.next().expect(msg);
    let fourth = interrupts.next().expect(msg);
    eprintln!("first={first:?}");
    eprintln!("second={second:?}");
    eprintln!("third={third:?}");
    eprintln!("fourth={fourth:?}");

    let max_prio = nvic.max_group_priority();
    eprintln!("max_prio={max_prio}");
    nvic.set_handler(first, first_handler);
    nvic.set_group_priority(first, 0);
    Nvic::enable(first);

    nvic.set_handler(second, second_handler);
    nvic.set_group_priority(second, 1);
    Nvic::enable(second);
    SECOND.store(second.into(), atomic::Ordering::Relaxed);

    nvic.set_handler(third, third_handler);
    nvic.set_group_priority(third, max_prio - 1);
    Nvic::enable(third);
    THIRD.store(third.into(), atomic::Ordering::Relaxed);

    nvic.set_handler(fourth, fourth_handler);
    nvic.set_group_priority(fourth, max_prio);
    Nvic::enable(fourth);
    FOURTH.store(fourth.into(), atomic::Ordering::Relaxed);

    Nvic::set_pending(first);
    nvic.enable_interrupts();

    assert_eq!(7, count());

    sh::exit()
}

static SECOND: AtomicU16 = AtomicU16::new(0);
static THIRD: AtomicU16 = AtomicU16::new(0);
static FOURTH: AtomicU16 = AtomicU16::new(0);

extern "C" fn first_handler() {
    assert_eq!(0, count());
    let second = ExternalInterrupt(SECOND.load(atomic::Ordering::Relaxed));
    Nvic::set_pending(second);
    assert_eq!(6, count());
}

extern "C" fn second_handler() {
    assert_eq!(1, count());
    let third = ExternalInterrupt(THIRD.load(atomic::Ordering::Relaxed));
    Nvic::set_pending(third);
    assert_eq!(5, count());
}

extern "C" fn third_handler() {
    assert_eq!(2, count());
    let fourth = ExternalInterrupt(FOURTH.load(atomic::Ordering::Relaxed));
    Nvic::set_pending(fourth);
    assert_eq!(4, count());
}

extern "C" fn fourth_handler() {
    assert_eq!(3, count());
}

fn count() -> usize {
    static COUNT: AtomicUsize = AtomicUsize::new(0);

    COUNT.fetch_add(1, atomic::Ordering::Relaxed)
}

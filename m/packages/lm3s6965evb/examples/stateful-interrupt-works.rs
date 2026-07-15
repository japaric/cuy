//! stateful interrupt works
//@ runner: $RUNNER
//@ target: $TARGET

#![no_std]
#![no_main]

use core::sync::atomic::{self, AtomicUsize};

use m_rt::interrupt_handler;
use m_rt::nvic::{HandlerState, Nvic};

m_rt::entry!(main);

fn main() -> ! {
    let nvic = Nvic::acquire();

    let mut interrupts = nvic.interrupts();
    let first = interrupts.next().unwrap();

    let handler = interrupt_handler!(MyHandler);
    nvic.set_stateful_handler(first, handler, MyHandler { count: 42 });
    Nvic::enable(first);
    Nvic::set_pending(first);
    nvic.enable_interrupts();
    assert_eq!(1, count());

    Nvic::set_pending(first);
    assert_eq!(3, count());

    sh::exit()
}

struct MyHandler {
    count: usize,
}

impl HandlerState for MyHandler {
    fn on_interrupt(&mut self) {
        let expected = match count() {
            0 => 42,
            2 => 43,
            _ => unreachable!(),
        };
        assert_eq!(expected, self.count);

        self.count += 1;
    }
}

fn count() -> usize {
    static COUNT: AtomicUsize = AtomicUsize::new(0);

    COUNT.fetch_add(1, atomic::Ordering::Relaxed)
}

//! can measure max stack usage
//@ runner: $RUNNER
//@ target: $TARGET

#![no_std]
#![no_main]

use m_rt::nvic::Nvic;
use sh::eprintln;

m_rt::entry!(main);

fn main() -> ! {
    let nvic = Nvic::acquire();

    let num_interrupts = nvic.num_interrupts();
    eprintln!("num_interrupts={}", num_interrupts);
    eprintln!("num_prio_bits={}", nvic.num_priority_bits());

    let mut count = 0;
    for int in nvic.interrupts() {
        assert!(nvic.is_implemented(int));
        count += 1;
    }
    assert_eq!(num_interrupts, count);

    sh::exit()
}

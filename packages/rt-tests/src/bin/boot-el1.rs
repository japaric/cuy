//@ boot-el1

#![no_std]
#![no_main]

use regs::CurrentEL;

rt::entry!(main);

fn main() -> ! {
    assert_eq!(CurrentEL::EL1, CurrentEL::read());

    sh::exit()
}

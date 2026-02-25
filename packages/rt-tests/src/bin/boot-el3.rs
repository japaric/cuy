//@ boot-el3

#![no_std]
#![no_main]

use regs::CurrentEL;

rt::entry!(main);

fn main() -> ! {
    assert_eq!(CurrentEL::EL3, CurrentEL::read());

    sh::exit()
}

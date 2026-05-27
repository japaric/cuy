//! An example application used for quick experimentation

#![no_std]
#![no_main]

use regs::CurrentEL;
use rt::StackMemory;
use sh::println;

rt::entry!(crate::main);

fn main() -> ! {
    println!("starting at {:?}", CurrentEL::read());

    rt::drop_el(at_el1, StackMemory::reserve(1.try_into().unwrap()));
}

extern "C" fn at_el1() -> ! {
    println!("dropped to {:?}", CurrentEL::read());

    sh::exit()
}

//! check that `.stack.boot` does indeed represent the call stack of the boot processor
//@ runner: $RUNNER
//@ target: $TARGET

#![no_std]
#![no_main]

use a_regs::SP;
use a_rt::LinkerSection;

a_rt::entry!(main);

fn main() -> ! {
    let sp = SP::read();

    let boot_stack = LinkerSection::boot_stack();
    assert!(boot_stack.contains(sp));

    sh::exit()
}

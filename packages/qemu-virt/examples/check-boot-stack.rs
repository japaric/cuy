//! check that `.stack.boot` does indeed represent the call stack of the boot processor
// runner: $RUNNER

#![no_std]
#![no_main]

use regs::SP;
use rt::LinkerSection;

rt::entry!(main);

fn main() -> ! {
    let sp = SP::read();

    let boot_stack = LinkerSection::boot_stack();
    assert!(boot_stack.contains(sp));

    sh::exit()
}

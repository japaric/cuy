//! check that `.stack.boot` does indeed represent the call stack of the boot processor
// runner: qemu-system-aarch64 -m 128 -cpu cortex-a53 -machine virt -nographic -semihosting -kernel

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

//! check that the startup code can boot in EL2
// runner: qemu-system-aarch64 -m 128 -cpu cortex-a53 -machine virt,virtualization=on -nographic -semihosting -kernel

#![no_std]
#![no_main]

use regs::CurrentEL;

rt::entry!(main);

fn main() -> ! {
    assert_eq!(
        CurrentEL::EL2,
        CurrentEL::read(),
        "wrong runner configuration for EL2"
    );

    sh::exit()
}

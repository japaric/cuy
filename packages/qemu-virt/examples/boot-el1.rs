// runner: qemu-system-aarch64 -cpu cortex-a53 -machine virt -nographic -semihosting -kernel

#![no_std]
#![no_main]

use regs::CurrentEL;

rt::entry!(main);

fn main() -> ! {
    assert_eq!(
        CurrentEL::EL1,
        CurrentEL::read(),
        "wrong runner configuration for EL1"
    );

    sh::exit()
}

// runner: qemu-system-aarch64 -cpu cortex-a53 -machine virt,secure=on -nographic -semihosting -kernel

#![no_std]
#![no_main]

use regs::CurrentEL;

rt::entry!(main);

fn main() -> ! {
    assert_eq!(
        CurrentEL::EL3,
        CurrentEL::read(),
        "wrong runner configuration for EL3"
    );

    sh::exit()
}

// runner: qemu-system-aarch64 -cpu neoverse-v1 -machine virt,virtualization=on -nographic -semihosting -kernel

#![no_std]
#![no_main]

use regs::CurrentEL;

rt::entry!(main);

fn main() -> ! {
    assert_eq!(CurrentEL::EL2, CurrentEL::read());

    sh::exit()
}

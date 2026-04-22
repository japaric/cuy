// FIXME does not work on earlier Cortex-A cores
// runner: qemu-system-aarch64 -cpu cortex-a76 -machine virt,virtualization=on -nographic -semihosting -kernel

#![no_std]
#![no_main]

use regs::CurrentEL;
use rt::StackMemory;

rt::entry!(main);

fn main() -> ! {
    assert_eq!(
        CurrentEL::EL2,
        CurrentEL::read(),
        "this example must start in EL2"
    );

    rt::drop_el(at_el1, StackMemory::reserve(1.try_into().unwrap()));
}

extern "C" fn at_el1() -> ! {
    assert_eq!(CurrentEL::EL1, CurrentEL::read());

    sh::exit()
}

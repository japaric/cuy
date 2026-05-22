//! check that EL2 to EL1 transition works
// FIXME does not work on earlier Cortex-A cores
//@ runner: $RUNNER_EL2_NOKERNEL -cpu cortex-a76 -kernel
//@ target: $TARGET

#![no_std]
#![no_main]

use a_regs::CurrentEL;
use a_rt::stack_memory;

a_rt::entry!(main);

fn main() -> ! {
    assert_eq!(
        CurrentEL::EL2,
        CurrentEL::read(),
        "this example must start in EL2"
    );

    a_rt::drop_el(at_el1, stack_memory!(4 * 1024).unwrap());
}

extern "C" fn at_el1() -> ! {
    assert_eq!(CurrentEL::EL1, CurrentEL::read());

    sh::exit()
}

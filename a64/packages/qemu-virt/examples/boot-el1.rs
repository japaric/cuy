//! check that the startup code can boot in EL1
//@ runner: $RUNNER_EL1
//@ target: $TARGET

#![no_std]
#![no_main]

use a64_regs::CurrentEL;

a64_rt::entry!(main);

fn main() -> ! {
    assert_eq!(
        CurrentEL::EL1,
        CurrentEL::read(),
        "wrong runner configuration for EL1"
    );

    sh::exit()
}

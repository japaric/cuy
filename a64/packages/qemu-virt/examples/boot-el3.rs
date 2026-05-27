//! check that the startup code can boot in EL3
//@ runner: $RUNNER_EL3
//@ target: $TARGET

#![no_std]
#![no_main]

use a64_regs::CurrentEL;

a64_rt::entry!(main);

fn main() -> ! {
    assert_eq!(
        CurrentEL::EL3,
        CurrentEL::read(),
        "wrong runner configuration for EL3"
    );

    sh::exit()
}

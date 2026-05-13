//! check that the startup code can boot in EL1
// runner: $RUNNER_EL1

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

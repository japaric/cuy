//! check that the startup code can boot in EL2
// runner: $RUNNER_EL2

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

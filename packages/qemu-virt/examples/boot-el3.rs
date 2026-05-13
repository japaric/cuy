//! check that the startup code can boot in EL3
// runner: $RUNNER_EL3

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

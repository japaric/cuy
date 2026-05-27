//! REQ001: check that the FPU can be used without raising an exception
//@ runner: $RUNNER_EL1
//@ target: $TARGET

#![no_std]
#![no_main]

use core::arch::asm;

use a64_regs::CurrentEL;

a64_rt::entry!(main);

fn main() -> ! {
    assert_eq!(
        CurrentEL::EL1,
        CurrentEL::read(),
        "this example must run in EL1"
    );
    let x = 1f64;
    let y = 2f64;

    let z: f64;
    // use assembly to avoid computation at compile time
    // SAFETY: pure operation that only involves CPU registers
    unsafe {
        asm!("fadd {:d}, {:d}, {:d}", out(vreg) z, in(vreg) x, in(vreg) y);
    }
    assert_eq!(3., z);

    sh::exit()
}

//! REQ001: check that the FPU can be used without raising an exception
// runner: qemu-system-aarch64 -m 128 -cpu cortex-a53 -machine virt -nographic -semihosting -kernel

#![no_std]
#![no_main]

use core::arch::asm;

use regs::CurrentEL;

rt::entry!(main);

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

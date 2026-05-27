//! REQ000: check that the .bss section is zeroed by the startup code
//@ runner: $RUNNER
//@ target: $TARGET

#![no_std]
#![no_main]

use core::sync::atomic::AtomicU64;

a64_rt::entry!(main);

static IN_BSS: AtomicU64 = AtomicU64::new(0);

fn main() -> ! {
    // use a volatile operation to prevent const evaluation
    // SAFETY: no concurrent access to this memory location
    let value = unsafe { IN_BSS.as_ptr().read_volatile() };
    // REQ000
    assert_eq!(0, value);

    sh::exit()
}

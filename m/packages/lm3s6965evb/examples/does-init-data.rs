//! REQ003 does initialize the `.data` section
//@ runner: $RUNNER
//@ target: $TARGET

#![no_std]
#![no_main]

use core::sync::atomic::AtomicU32;

const EXPECTED: u32 = 1;
static X: AtomicU32 = AtomicU32::new(EXPECTED);

m_rt::entry!(main);

fn main() -> ! {
    // use volatile to force a LDR instruction and avoid compile-time evaluation
    // SAFETY: read-only access
    let got = unsafe { X.as_ptr().read_volatile() };
    assert_eq!(EXPECTED, got);

    sh::exit()
}

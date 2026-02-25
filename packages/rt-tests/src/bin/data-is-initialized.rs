#![no_std]
#![no_main]

use core::sync::atomic::{self, AtomicU64};

rt::entry!(main);

const EXPECTED: u64 = 1;
static IN_DATA: AtomicU64 = AtomicU64::new(EXPECTED);

fn main() -> ! {
    let old = IN_DATA.fetch_add(1, atomic::Ordering::SeqCst);
    assert_eq!(EXPECTED, old);

    sh::exit()
}

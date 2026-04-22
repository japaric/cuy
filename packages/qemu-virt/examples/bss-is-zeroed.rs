// REQ000
// runner: qemu-system-aarch64 -cpu cortex-a53 -machine virt -nographic -semihosting -kernel

#![no_std]
#![no_main]

use core::sync::atomic::{self, AtomicU64};

rt::entry!(main);

const EXPECTED: u64 = 0;
static IN_BSS: AtomicU64 = AtomicU64::new(EXPECTED);

fn main() -> ! {
    let old = IN_BSS.fetch_add(1, atomic::Ordering::SeqCst);
    // REQ000
    assert_eq!(EXPECTED, old);

    sh::exit()
}

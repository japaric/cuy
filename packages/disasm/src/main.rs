//! A minimal application used in binary analysis

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
fn _start(bss_start: *mut u64, bss_end: *mut u64) {
    let mut current = bss_start;
    while current < bss_end {
        unsafe {
            current.write_volatile(0);
            current = current.add(1);
        }
    }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

//! A minimal application used in binary analysis

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
fn _start(_bss_lower: *mut u64, _bss_higher: *mut u64) {
    let mut current = _bss_lower;
    while current < _bss_higher {
        unsafe {
            current.write_volatile(0);
            current = current.add(1);
        }
    }

    unsafe { core::arch::asm!("nop") }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

//! A minimal application used in binary analysis

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
extern "C" fn _start(lower: *mut u32, higher: *mut u32, color: u32) {
    let mut curr = lower;
    while curr < higher {
        unsafe {
            curr.write_volatile(color);

            curr = curr.add(1);
        }
    }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

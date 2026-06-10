//! A minimal application used in binary analysis

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
fn _start(lma: *const u64, vma_lower: *mut u64, vma_higher: *mut u64) -> ! {
    let mut from = lma;
    let mut to = vma_lower;
    while to < vma_higher {
        unsafe {
            let value = from.read_volatile();
            to.write_volatile(value);

            from = from.add(1);
            to = to.add(1);
        }
    }

    loop {}
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

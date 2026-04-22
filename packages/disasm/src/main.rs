//! A minimal application used in binary analysis

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
fn _start(x: f64, y: f64) -> f64 {
    x + y
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

//! can override default exception handler
//@ runner: $RUNNER
//@ target: $TARGET

#![no_std]
#![no_main]

use core::arch::asm;

use m_rt::vtor::{SystemInterrupt, VectActive};

m_rt::entry!(main);

fn main() -> ! {
    SystemInterrupt::SVCall.set_handler(handler as extern "C" fn());
    // trigger the SVC handler
    // SAFETY: a handler is always installed by default
    unsafe { asm!("SVC 0x00") }

    panic!("returned from SVC handler")
}

extern "C" fn handler() {
    assert_eq!(
        VectActive::SystemInterrupt(SystemInterrupt::SVCall),
        VectActive::get()
    );

    sh::exit()
}

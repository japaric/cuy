//! can set default exception handler
//@ runner: $RUNNER
//@ target: $TARGET

#![no_std]
#![no_main]

use core::arch::asm;

use m_rt::vtor::Exception;

m_rt::entry!(main);

fn main() -> ! {
    Exception::SVCall.set_handler(handler);
    // trigger the SVC handler
    // SAFETY: a handler is always installed by default
    unsafe { asm!("SVC 0x00") }

    panic!("returned from SVC handler")
}

extern "C" fn handler() {
    sh::exit()
}

//! Exception handler can inspect registers pushed onto the stack on exception entry
//@ runner: $RUNNER
//@ target: $TARGET

#![no_std]
#![no_main]

use core::arch::asm;
use core::sync::atomic::{self, AtomicBool};

use m_rt::exception_handler_with_stacked_registers;
use m_rt::vtor::{StackedRegisters, SystemInterrupt, VectActive};

m_rt::entry!(main);

static TOOK_EXCEPTION: AtomicBool = AtomicBool::new(false);

fn main() -> ! {
    SystemInterrupt::SVCall.set_handler(exception_handler_with_stacked_registers!(handler));
    // trigger a SVCall exception
    // SAFETY: a handler is always installed by default
    unsafe { asm!("SVC 0", in("r0") 0, in("r1") 1, in("r2") 2, in("r12") 12) }

    assert!(TOOK_EXCEPTION.load(atomic::Ordering::Relaxed));

    sh::exit()
}

extern "C" fn handler(state: &StackedRegisters) {
    const SVC_0: u16 = 0xdf00;

    assert_eq!(
        VectActive::SystemInterrupt(SystemInterrupt::SVCall),
        VectActive::get()
    );
    assert_eq!(0, state.r0);
    assert_eq!(1, state.r1);
    assert_eq!(2, state.r2);
    assert_eq!(12, state.r12);
    // SAFETY: valid program counter as per ISA; `SVC` is always 16-bit encoded
    let trigger_insn = unsafe {
        (state.return_address as *const u16)
            .offset(-1)
            .read_volatile()
    };
    assert_eq!(SVC_0, trigger_insn);

    TOOK_EXCEPTION.store(true, atomic::Ordering::Relaxed);
}

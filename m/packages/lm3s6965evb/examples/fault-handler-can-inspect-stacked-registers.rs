//! Fault handler can inspect registers pushed onto the stack on exception entry
//@ runner: $RUNNER
//@ target: $TARGET

#![no_std]
#![no_main]

use core::arch::asm;

use m_rt::fault_handler_with_stacked_registers;
use m_rt::vtor::{NonMaskableFault, StackedRegisters};

m_rt::entry!(main);

fn main() -> ! {
    // SAFETY: even if semihosting uses a static variable, there's no problematic concurrency in
    // this program
    unsafe {
        NonMaskableFault::HardFault.set_handler(fault_handler_with_stacked_registers!(handler));
    }
    // trigger a HardFault exception
    // SAFETY: a handler is always installed by default
    unsafe { asm!("UDF 0", in("r0") 0, in("r1") 1, in("r2") 2, in("r12") 12) }

    panic!("returned from HardFault handler")
}

extern "C" fn handler(state: &StackedRegisters) -> ! {
    const UDF_0: u16 = 0xde00;

    assert_eq!(0, state.r0);
    assert_eq!(1, state.r1);
    assert_eq!(2, state.r2);
    assert_eq!(12, state.r12);
    // SAFETY: valid program counter as per ISA
    let trigger_insn = unsafe { (state.return_address as *const u16).read_volatile() };
    assert_eq!(UDF_0, trigger_insn);

    sh::exit()
}

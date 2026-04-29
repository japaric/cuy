use core::arch::{asm, naked_asm};

use regs::{CurrentEL, DAIF, ELR_EL2, SPSR_EL2};

use crate::StackMemory;

/// Executes `f` at one lower EL using the given `stack_memory`
///
/// `f` will inherit the current interrupt mask bits (DAIF)
pub fn drop_el(f: extern "C" fn() -> !, stack_memory: StackMemory) -> ! {
    #[unsafe(naked)]
    extern "C" fn trampoline(f: usize, initial_sp: usize) -> ! {
        naked_asm!("mov SP, x1", "br x0")
    }

    // TODO this probably should only be allowed to be called at most once?
    // as in probably we don't want this to be called from an exception handler

    match CurrentEL::read() {
        CurrentEL::EL0 => panic!("already at EL0"),

        CurrentEL::EL1 => todo!(),

        CurrentEL::EL2 => {
            ELR_EL2::write(trampoline as *const () as usize);

            let daif = DAIF::read() as u32;
            let m = 0b0101; // EL1 with SP_EL1
            let spsr = m | daif;
            SPSR_EL2::write(spsr);
        }

        CurrentEL::EL3 => todo!(),
    }

    // SAFETY: ELR and SPSR has been set to sensible values
    unsafe {
        asm!(
            "eret",
            in("x0") f as *const () as usize,
            in("x1") stack_memory.higher(),
            options(nomem, noreturn),
        )
    }
}

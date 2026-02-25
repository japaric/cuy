//! AArch64 startup code

#![no_std]

use core::arch::{asm, global_asm, naked_asm};
use core::num::NonZeroUsize;
use core::ptr;
use core::sync::atomic::{self, AtomicUsize};

use regs::{CurrentEL, DAIF, ELR_EL2, SPSR_EL2};

/// Defines the entry point of a program
#[macro_export]
macro_rules! entry {
    ($path:path) => {
        const _: () = {
            #[unsafe(export_name = "main")]
            extern "C" fn __implementation_detail__() -> ! {
                ($path as fn() -> !)()
            }
        };
    };
}

global_asm!(include_str!("start.s"));

#[unsafe(no_mangle)]
extern "C" fn rust_start() -> ! {
    unsafe extern "C" {
        fn main() -> !;
    }

    // SAFETY: called exactly once
    unsafe { main() }
}

/// Page-sized call stack memory
///
/// Backed by a lock-free memory allocator
pub struct StackMemory {
    offset: usize,
    num_pages: NonZeroUsize,
}

impl StackMemory {
    const PAGE_SIZE: usize = 4 * 1024;

    /// Reserves `num_pages` of stack memory
    ///
    /// One page is 4 KiB
    pub fn reserve(num_pages: NonZeroUsize) -> Self {
        static OFFSET: AtomicUsize = AtomicUsize::new(0);

        let offset = OFFSET.fetch_add(num_pages.get(), atomic::Ordering::AcqRel);
        // TODO check for OOM

        Self { offset, num_pages }
    }

    fn address(&self) -> usize {
        unsafe extern "C" {
            static _boot_stack_end: u8;
        }

        ptr::addr_of!(_boot_stack_end) as usize
            + Self::PAGE_SIZE * (self.offset + self.num_pages.get())
    }
}

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
            in("x1") stack_memory.address(),
            options(nomem, noreturn),
        )
    }
}

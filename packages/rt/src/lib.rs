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
        $crate::entry!($path, stack_size = 16 * 1024);
    };
    ($path:path, stack_size=$stack_size:expr) => {
        const _: () = {
            #[unsafe(export_name = "main")]
            extern "C" fn __implementation_detail__() -> ! {
                ($path as fn() -> !)()
            }

            #[unsafe(link_section = ".stack.boot")]
            #[used]
            static __BOOT_STACK: [u8; $stack_size] = [0; $stack_size];
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
            static _stack_higher: u8;
        }

        ptr::addr_of!(_stack_higher) as usize
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

/// An output linker section
#[derive(Debug, PartialEq)]
pub struct LinkerSection {
    lower: usize,
    higher: usize,
}

impl LinkerSection {
    /// `.text` section
    pub fn text() -> Self {
        unsafe extern "C" {
            static _text_lower: u8;
            static _text_higher: u8;
        }

        Self {
            lower: &raw const _text_lower as usize,
            higher: &raw const _text_higher as usize,
        }
    }

    /// `.rodata` section
    pub fn rodata() -> Self {
        unsafe extern "C" {
            static _rodata_lower: u8;
            static _rodata_higher: u8;
        }

        Self {
            lower: &raw const _rodata_lower as usize,
            higher: &raw const _rodata_higher as usize,
        }
    }

    /// `.data` section
    pub fn data() -> Self {
        unsafe extern "C" {
            static _data_lower: u8;
            static _data_higher: u8;
        }

        Self {
            lower: &raw const _data_lower as usize,
            higher: &raw const _data_higher as usize,
        }
    }

    /// `.bss` section
    pub fn bss() -> Self {
        unsafe extern "C" {
            static _bss_lower: u8;
            static _bss_higher: u8;
        }

        Self {
            lower: &raw const _bss_lower as usize,
            higher: &raw const _bss_higher as usize,
        }
    }

    /// `.stack` section
    pub fn stack() -> Self {
        unsafe extern "C" {
            static _stack_lower: u8;
            static _stack_higher: u8;
        }

        Self {
            lower: &raw const _stack_lower as usize,
            higher: &raw const _stack_higher as usize,
        }
    }

    /// Returns the lower address boundary of the linker section
    pub fn lower(&self) -> usize {
        self.lower
    }

    /// Returns the higher address boundary of the linker section
    pub fn higher(&self) -> usize {
        self.higher
    }

    /// Returns the size of the linker section in bytes
    pub fn size(&self) -> usize {
        self.higher - self.lower
    }
}

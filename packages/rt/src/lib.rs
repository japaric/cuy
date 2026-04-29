//! AArch64 startup code

#![no_std]

use core::num::NonZeroUsize;
use core::ptr;
use core::sync::atomic::{self, AtomicUsize};

pub use el::drop_el;
pub use linker_section::LinkerSection;

mod el;
mod entry;
mod linker_section;

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

use core::ops;

use crate::LinkerSection;

/// An ownning pointer into statically allocated call stack memory
///
/// This memory is obtained using the `stack_memory!` macro
pub struct StackMemory {
    section: LinkerSection,
}

impl ops::Deref for StackMemory {
    type Target = LinkerSection;

    fn deref(&self) -> &Self::Target {
        &self.section
    }
}

impl StackMemory {
    #[doc(hidden)]
    pub unsafe fn new(lower: usize, higher: usize) -> Self {
        Self {
            section: LinkerSection::new(lower, higher),
        }
    }
}

/// Statically allocates stack memory and hands out an owning pointer
///
/// This macro returns `Option<StackMemory>`; the `Some` variant is returned only on
/// the first invocation
#[macro_export]
macro_rules! stack_memory {
    ($size:expr) => {{
        static TAKEN: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

        // this atomic operation does not synchronize access to shared memory; it instead hands
        // out an owning pointer to memory so no atomic barrier is needed
        if !TAKEN.swap(true, core::sync::atomic::Ordering::Relaxed) {
            #[repr(align(64))]
            #[repr(C)]
            struct Stack([u8; $size]);

            #[unsafe(link_section = ".stack")]
            static mut STACK: Stack = Stack([0; $size]);

            const _: () = assert!($size > 0);

            let lower = &raw const STACK as usize;
            let higher = lower + core::mem::size_of::<Stack>();
            Some(unsafe { $crate::StackMemory::new(lower, higher) })
        } else {
            None
        }
    }};
}

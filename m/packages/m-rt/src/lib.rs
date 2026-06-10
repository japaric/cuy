//! Cortex-M startup code

#![no_std]

use core::arch::global_asm;

pub use linker_section::LinkerSection;

mod linker_section;

/// Defines the user entry point of a program
///
/// Optionally the size of the boot core call stack can be specified; the default value is 16 KiB.
/// Due to alignment requirements, the stack size may be rounded up.
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

// called by `_start` in `start.s`
#[unsafe(no_mangle)]
extern "C" fn rust_start() -> ! {
    unsafe extern "C" {
        fn main() -> !;
    }

    // SAFETY: signature is enforced by `entry!` macro
    unsafe { main() }
}

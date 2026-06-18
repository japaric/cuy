//! Cortex-M startup code

#![no_std]

use core::arch::global_asm;

pub use linker_section::LinkerSection;

mod linker_section;
pub mod vtor;

/// Defines the user entry point of a program
///
/// Optionally the size of the boot core call stack can be specified; the default value is 16 KiB.
/// Due to alignment requirements, the stack size may be rounded up.
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

            #[unsafe(link_section = ".stack")]
            #[used]
            static __STACK: [u8; $stack_size] = [0; $stack_size];
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

    vtor::set();

    // SAFETY: signature is enforced by `entry!` macro
    unsafe { main() }
}

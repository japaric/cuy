use core::arch::global_asm;

use crate::vtor;

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

global_asm!("
  .section .text._start
  .global _start
_start:
  /* REQ000: zero bss */
  /* NOTE: this assumes the section is 8-byte aligned (REQ006) */
  ldr r0, =_bss_lower
  ldr r1, =_bss_higher
  bl {zero_bss}

  /* REQ003: initialize the .data section */
  /* NOTE: this assumes that the section LMA and VMA are 8-byte aligned (REQ007) */
  ldr  r0, =_data_lma
  ldr  r1, =_data_lower
  ldr  r2, =_data_higher
  bl {init_data}

  /* jump into the Rust part of the entry point */
  b {rust_start}
", zero_bss = sym zero_bss, init_data = sym init_data, rust_start = sym rust_start);

/// # Safety
/// - `lower..higher` must be the boundaries of a linker section that needs runtime zeroing, like
/// `.bss`
/// - this function must be called before any Rust code is called
#[optimize::size]
unsafe extern "C" fn zero_bss(lower: *mut u64, higher: *mut u64) {
    let mut curr = lower;
    while curr < higher {
        // SAFETY: valid location as per function's Safety Requirements and called before any
        // Rust is called
        unsafe { curr.write_volatile(0) };
        // SAFETY: within bounds given `curr < higher` checks and function's Safety Requirements
        curr = unsafe { curr.add(1) };
    }
}

/// # Safety
/// - `lower..higher` must point to the VMA boundaries of a linker section that needs runtime
///   initialization, like `.data`
/// - `lma..(lma+(higher-lower))` must point to the LMA boundaries of said section
/// - this function must be called before any Rust code is called
#[optimize::size]
unsafe extern "C" fn init_data(lma: *const u64, lower: *mut u64, higher: *mut u64) {
    let mut from = lma;
    let mut to = lower;
    while to < higher {
        // SAFETY: valid memory location given function's Safety Requirements
        let value = unsafe { from.read_volatile() };
        // SAFETY: valid memory location given function's Safety Requirements
        unsafe { to.write_volatile(value) };
        // SAFETY: within bounds as per function's Safety Requirements
        from = unsafe { from.add(1) };
        // SAFETY: within bounds as per function's Safety Requirements
        to = unsafe { to.add(1) };
    }
}

/// # Safety
/// must only be called only once
unsafe extern "C" fn rust_start() -> ! {
    unsafe extern "C" {
        fn main() -> !;
    }

    vtor::set();

    // SAFETY: signature is enforced by `entry!` macro
    unsafe { main() }
}

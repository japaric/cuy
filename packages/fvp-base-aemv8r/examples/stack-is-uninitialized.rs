//! check that the .stack section is left unitialized by the ELF loader
// runner: $RUNNER

#![no_std]
#![no_main]

use rt::stack_memory;

rt::entry!(main);

// from `FVP_BaseR_AEMv8R --list-params`
const FILL_PATTERN_1: u32 = 0xdfdfdfcf;
const FILL_PATTERN_2: u32 = 0xcfdfdfdf;

fn main() -> ! {
    let stack = stack_memory!(4 * 1024).unwrap();
    // use a volatile operation to prevent const evaluation
    // SAFETY: no concurrent access to this memory location
    let value = unsafe { (stack.lower() as *const u32).read_volatile() };
    assert!(value == FILL_PATTERN_1 || value == FILL_PATTERN_2);

    sh::exit()
}

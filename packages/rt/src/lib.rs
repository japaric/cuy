//! AArch64 startup code

#![no_std]

pub use el::drop_el;
pub use linker_section::LinkerSection;
pub use stack_memory::StackMemory;

mod el;
mod entry;
mod linker_section;
mod stack_memory;

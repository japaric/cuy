//! Cortex-M startup code

#![no_std]

pub use linker_section::LinkerSection;

mod entry;
mod linker_section;
pub mod nvic;
mod scs;
pub mod stack;
pub mod vtor;

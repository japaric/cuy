//! Cortex-M startup code

#![no_std]

pub use linker_section::LinkerSection;

mod entry;
mod linker_section;
pub mod vtor;

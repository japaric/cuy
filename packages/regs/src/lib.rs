//! AArch64 system registers

#![allow(non_camel_case_types)]
#![no_std]

use core::arch::asm;

/// Current Exception Level register
// [ARM-R64/C5.2.1]
#[derive(Debug, PartialEq)]
pub enum CurrentEL {
    /// Exception Level 0
    EL0,
    /// Exception Level 1
    EL1,
    /// Exception Level 2
    EL2,
    /// Exception Level 3
    EL3,
}

impl CurrentEL {
    /// Reads the CurrentEL register
    pub fn read() -> Self {
        let value: u64;
        // SAFETY: no side effects
        unsafe {
            asm!(
                "MRS {}, CurrentEL",
                out(reg) value,
                options(nomem, pure),
            )
        }
        match (value >> 2) & 0b11 {
            0b00 => Self::EL0,
            0b01 => Self::EL1,
            0b10 => Self::EL2,
            _ => Self::EL3,
        }
    }
}

/// Interrupt Mask Bits
// [ARM-R64/C5.2.2]
pub struct DAIF;

impl DAIF {
    /// Reads the Interrupt Mask Bits
    pub fn read() -> u16 {
        let value: u64;
        // SAFETY: no side effects
        unsafe {
            asm!(
                "MRS {}, DAIF",
                out(reg) value,
                options(nomem, pure),
            )
        }
        value as u16
    }
}

/// Saved Program Status Register (EL2)
// [ARM-R64/C5.2.15]
pub struct SPSR_EL2;

impl SPSR_EL2 {
    /// Writes to the SPSR
    pub fn write(value: u32) {
        // SAFETY: not unsafe by itself; `eret` is the dangerous operation
        unsafe {
            asm!(
                "MSR SPSR_EL2, {}",
                in(reg) value as u64,
                options(nomem),
            )
        }
    }
}

/// Exception Link Regsiter (EL2)
// [ARM-R64/C5.2.5]
pub struct ELR_EL2;

impl ELR_EL2 {
    /// Writes to the ELR
    pub fn write(value: usize) {
        // SAFETY: not unsafe by itself; `eret` is the dangerous operation
        unsafe {
            asm!(
                "MSR ELR_EL2, {}",
                in(reg) value ,
                options(nomem),
            )
        }
    }
}

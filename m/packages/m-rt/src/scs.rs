use mmio::{RwReg, SafeRoReg};

const SCS: usize = 0xE000_ED00;
// SAFETY: cross checked against TRM
pub const ICSR: SafeRoReg<SCS, usize> = unsafe { SafeRoReg::new(0x4) };
// SAFETY: cross checked against TRM
pub const VTOR: RwReg<SCS, usize> = unsafe { RwReg::new(0x8) };
// SAFETY: cross checked against TRM
pub const AIRCR: SafeRoReg<SCS, u32> = unsafe { SafeRoReg::new(0xC) };

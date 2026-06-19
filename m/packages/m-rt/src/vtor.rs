//! Vector table manipulation

use core::sync::atomic;
use core::sync::atomic::AtomicPtr;

use mmio::RwReg;

// Includes initial stack pointer entry
const NUM_EXCEPTIONS: usize = 16;
// FIXME other cores support more than 240 device interrupts
pub(crate) const NUM_INTERRUPTS: usize = 240;

const SCS: usize = 0xE000_ED00;
// SAFETY: cross checked against TRM
const SCS_VTOR: RwReg<SCS, usize> = unsafe { RwReg::new(0x8) };

#[repr(C)]
// for 256 entries we need 512-word alignment
#[repr(align(2048))]
// TODO size should be configurable
// 16 exceptions + 240 device interrupts (for Cortex-M3)
struct Entries([AtomicPtr<()>; NUM_EXCEPTIONS + NUM_INTERRUPTS]);

static ENTRIES: Entries = Entries([const { AtomicPtr::new(unhandled as *mut ()) }; 256]);

// TODO report exception number
// TODO report stacked registers
extern "C" fn unhandled() -> ! {
    panic!("unhandled exception")
}

pub(crate) fn set() {
    // SAFETY: alignment requirements are satisfied; entries are set
    unsafe { SCS_VTOR.write(&raw const ENTRIES as usize) }
}

/// Signature of fault handler
pub type FaultHandler = extern "C" fn() -> !;

/// Signature of exception handler
pub type ExceptionHandler = extern "C" fn();

/// Fault exceptions that cannot be masked, e.g. using CPSID or BASEPRI
#[derive(Clone, Copy)]
#[non_exhaustive]
pub enum NonMaskableFault {
    /// The NonMaskable Interrupt
    Nmi = 2,
    /// Hard Fault exception
    HardFault = 3,
}

impl NonMaskableFault {
    /// Sets a handler for this non-maskable fault
    ///
    /// # Safety
    ///
    /// These faults cannot be masked so they'll break critical sections based on disabling/masking
    /// interrupts; the handler must be careful when accessing shared memory, e.g. static variables
    pub unsafe fn set_handler(&self, f: FaultHandler) {
        ENTRIES.0[*self as usize].store(f as *mut (), atomic::Ordering::Relaxed);
    }
}

/// Fault exceptions that can be masked, e.g. using CPSID or BASEPRI
#[derive(Clone, Copy)]
#[non_exhaustive]
pub enum Fault {
    /// Memory Management fault
    MemManage = 4,
    /// Bus fault
    BusFault = 5,
    /// Usage fault
    UsageFault = 6,
}

impl Fault {
    /// Sets a handler for this maskable fault
    pub fn set_handler(&self, f: FaultHandler) {
        ENTRIES.0[*self as usize].store(f as *mut (), atomic::Ordering::Relaxed);
    }
}

/// Non-fault exceptions
#[derive(Clone, Copy)]
#[non_exhaustive]
pub enum Exception {
    /// SuperVisor Call exception
    SVCall = 11,
    /// PendSV exception
    PendSV = 14,
    /// System timer exception
    SysTick = 15,
}

impl Exception {
    /// Sets a handler for this exception
    pub fn set_handler(&self, f: ExceptionHandler) {
        ENTRIES.0[*self as usize].store(f as *mut (), atomic::Ordering::Relaxed);
    }
}

// TODO API to set interrupt handler

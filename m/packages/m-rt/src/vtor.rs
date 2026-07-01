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

/// A function that can be used as a fault handler
///
/// # Safety
/// Must not be implemented manually; use the `fault_handler_with_stacked_registers!` macro or
/// use a function pointer with type `extern "C" fn() -> !`
pub unsafe trait FaultHandler {
    #[doc(hidden)]
    fn address(self) -> usize;
}

/// Returns a `FaultHandler` implementation
///
/// Takes a path to a function with signature `extern "C" fn(&StackedRegisters) -> !`. The
/// function will have access to the registers pushed onto the stack on exception entry
#[macro_export]
macro_rules! fault_handler_with_stacked_registers {
    ($path:path) => {{
        struct S;
        // function signature validation
        const _: extern "C" fn(&$crate::vtor::StackedRegisters) -> ! = $path;
        unsafe impl $crate::vtor::FaultHandler for S {
            fn address(self) -> usize {
                #[unsafe(naked)]
                extern "C" fn trampoline() -> ! {
                    core::arch::naked_asm!(
                        "mov r0, sp
b {}",
                        sym $path
                    )
                }

                trampoline as usize
            }
        }
        S
    }};
}

/// Registers pushed onto the stack on exception entry
#[repr(C)]
pub struct StackedRegisters {
    /// Processor register 0
    pub r0: usize,
    /// Processor register 1
    pub r1: usize,
    /// Processor register 2
    pub r2: usize,
    /// Processor register 3
    pub r3: usize,
    /// Processor register 12
    pub r12: usize,
    /// Link Register
    pub lr: usize,
    /// Return Address
    ///
    /// For *precise* faults, this is the PC location of the instruction that triggered the faul t
    pub return_address: usize,
    /// Program Status Register
    pub xpsr: usize,
}

/// Safety: matches the ABI expected by the ISA
unsafe impl FaultHandler for extern "C" fn() -> ! {
    fn address(self) -> usize {
        self as usize
    }
}

/// Signature of exception handler
pub type ExceptionHandler = extern "C" fn();

/// Fault exceptions that cannot be masked, e.g. using CPSID or BASEPRI
#[derive(Clone, Copy)]
#[non_exhaustive]
pub enum NonMaskableFault {
    /// The NonMaskable Interrupt
    NonMaskableInt = -14,
    /// Hard Fault exception
    HardFault = -13,
}

impl NonMaskableFault {
    /// Sets a handler for this non-maskable fault
    ///
    /// # Safety
    ///
    /// These faults cannot be masked so they'll break critical sections based on disabling/masking
    /// interrupts; the handler must be careful when accessing shared memory, e.g. static variables
    pub unsafe fn set_handler(&self, f: impl FaultHandler) {
        ENTRIES.0[(NUM_EXCEPTIONS as isize + *self as isize) as usize]
            .store(f.address() as *mut (), atomic::Ordering::Relaxed);
    }
}

/// Fault exceptions that can be masked, e.g. using CPSID or BASEPRI
#[derive(Clone, Copy)]
#[non_exhaustive]
pub enum Fault {
    /// Memory Management fault
    MemoryManagement = -12,
    /// Bus fault
    BusFault = -11,
    /// Usage fault
    UsageFault = -10,
    /// Secure fault
    SecureFault = -9,
}

impl Fault {
    /// Sets a handler for this maskable fault
    pub fn set_handler(&self, f: impl FaultHandler) {
        ENTRIES.0[(NUM_EXCEPTIONS as isize + *self as isize) as usize]
            .store(f.address() as *mut (), atomic::Ordering::Relaxed);
    }
}

/// Non-fault exceptions
#[derive(Clone, Copy)]
#[non_exhaustive]
pub enum Exception {
    /// SuperVisor Call exception
    SVCall = -5,
    /// SuperVisor Call exception
    DebugMonitor = -4,
    /// PendSV exception
    PendSV = -2,
    /// System timer exception
    SysTick = -1,
}

impl Exception {
    /// Sets a handler for this exception
    pub fn set_handler(&self, f: ExceptionHandler) {
        ENTRIES.0[(NUM_EXCEPTIONS as isize + *self as isize) as usize]
            .store(f as *mut (), atomic::Ordering::Relaxed);
    }
}

// TODO API to set interrupt handler

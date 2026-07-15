//! Vector table manipulation

use core::fmt;
use core::sync::atomic;
use core::sync::atomic::AtomicPtr;

use crate::scs;

// Includes initial stack pointer entry
const NUM_EXCEPTIONS: usize = 16;
pub(crate) const NUM_INTERRUPTS: usize = 496;

#[repr(C)]
// for 512 entries we need 2048-byte alignment
#[repr(align(2048))]
// TODO size should be configurable
// 16 exceptions + 496 device interrupts (for v8M.mainline)
struct Entries([AtomicPtr<()>; NUM_EXCEPTIONS + NUM_INTERRUPTS]);

static ENTRIES: Entries =
    Entries([const { AtomicPtr::new(unhandled as *mut ()) }; NUM_EXCEPTIONS + NUM_INTERRUPTS]);

/// The current executing exception
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VectActive {
    /// Thread mode
    ThreadMode,
    /// Non-maskable fault
    NonMaskableFault(NonMaskableFault),
    /// Maskable fault
    Fault(Fault),
    /// System interrupt
    SystemInterrupt(SystemInterrupt),
    /// External interrupt
    ExternalInterrupt(u16),
    /// Reserved exception number
    Reserved(u8),
}

impl VectActive {
    /// Returns `VectActive`
    pub fn get() -> Self {
        match scs::ICSR.read() & ((1 << 9) - 1) {
            0 => Self::ThreadMode,
            2 => Self::NonMaskableFault(NonMaskableFault::NonMaskableInt),
            3 => Self::NonMaskableFault(NonMaskableFault::HardFault),
            4 => Self::Fault(Fault::MemoryManagement),
            5 => Self::Fault(Fault::BusFault),
            6 => Self::Fault(Fault::UsageFault),
            7 => Self::Fault(Fault::SecureFault),
            11 => Self::SystemInterrupt(SystemInterrupt::SVCall),
            12 => Self::Fault(Fault::DebugMonitor),
            14 => Self::SystemInterrupt(SystemInterrupt::PendSV),
            15 => Self::SystemInterrupt(SystemInterrupt::SysTick),
            reserved if reserved < 16 => Self::Reserved(reserved as u8),
            interrupt => Self::ExternalInterrupt((interrupt.wrapping_sub(16)) as u16),
        }
    }
}

#[unsafe(naked)]
extern "C" fn unhandled() -> ! {
    extern "C" fn handler(context: &StackedRegisters) -> ! {
        panic!(
            "unhandled exception {:?}; context: {context:#?}",
            VectActive::get()
        )
    }

    core::arch::naked_asm!(
        "mov r0, sp
    b {}",
        sym handler
    )
}

pub(crate) fn set() {
    // SAFETY: alignment requirements are satisfied; entries are set
    unsafe { scs::VTOR.write(&raw const ENTRIES as usize) }
}

/// A function that can be used as an exception handler
///
/// # Safety
/// Must not be implemented manually; use the `exception_handler_with_stacked_registers!` macro or
/// use a function pointer with type `extern "C" fn()`
pub unsafe trait ExceptionHandler {
    #[doc(hidden)]
    fn address(self) -> usize;
}

/// Safety: matches the ABI expected by the ISA
unsafe impl ExceptionHandler for extern "C" fn() {
    fn address(self) -> usize {
        self as usize
    }
}

/// Returns an `ExceptionHandler` implementation
///
/// Takes a path to a function with signature `extern "C" fn(&StackedRegisters)`. The
/// function will have access to the registers pushed onto the stack on exception entry
#[macro_export]
macro_rules! exception_handler_with_stacked_registers {
    ($path:path) => {{
        struct S;
        // function signature validation
        const _: extern "C" fn(&$crate::vtor::StackedRegisters) = $path;
        unsafe impl $crate::vtor::ExceptionHandler for S {
            fn address(self) -> usize {
                #[unsafe(naked)]
                extern "C" fn trampoline() {
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

/// A function that can be used as a fault handler
///
/// # Safety
/// Must not be implemented manually; use the `fault_handler_with_stacked_registers!` macro or
/// use a function pointer with type `extern "C" fn() -> !`
pub unsafe trait FaultHandler {
    #[doc(hidden)]
    fn address(self) -> usize;
}

/// Safety: matches the ABI expected by the ISA
unsafe impl FaultHandler for extern "C" fn() -> ! {
    fn address(self) -> usize {
        self as usize
    }
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
// TODO FPU registers (which are lazily stacked)
#[non_exhaustive]
#[derive(Debug)]
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

/// Fault exceptions that cannot be masked, e.g. using CPSID or BASEPRI
#[derive(Clone, Copy, Debug, PartialEq)]
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
#[derive(Clone, Copy, Debug, PartialEq)]
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
    /// DebugMonitor exception
    DebugMonitor = -4,
}

impl Fault {
    /// Sets a handler for this maskable fault
    pub fn set_handler(&self, f: impl FaultHandler) {
        ENTRIES.0[(NUM_EXCEPTIONS as isize + *self as isize) as usize]
            .store(f.address() as *mut (), atomic::Ordering::Relaxed);
    }
}

/// Non-fault exceptions
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum SystemInterrupt {
    /// SuperVisor Call exception
    SVCall = -5,
    /// PendSV exception
    PendSV = -2,
    /// System timer exception
    SysTick = -1,
}

impl SystemInterrupt {
    /// Sets a handler for this exception
    pub fn set_handler(&self, f: impl ExceptionHandler) {
        ENTRIES.0[(NUM_EXCEPTIONS as isize + *self as isize) as usize]
            .store(f.address() as *mut (), atomic::Ordering::Relaxed);
    }
}

/// an external interrupt
#[derive(Clone, Copy)]
pub struct ExternalInterrupt {
    pub(crate) nr: u16,
}

/// creates an external interrupt
///
/// # Panics
/// - if `nr` exceeds the maximum number of interrupts supported by the ISA
#[allow(non_snake_case)]
pub fn ExternalInterrupt(nr: u16) -> ExternalInterrupt {
    assert!(usize::from(nr) < NUM_INTERRUPTS);
    ExternalInterrupt { nr }
}

impl ExternalInterrupt {
    pub(crate) fn set_handler(&self, f: extern "C" fn()) {
        ENTRIES.0[NUM_EXCEPTIONS + usize::from(self.nr)]
            .store(f as *mut (), atomic::Ordering::Relaxed);
    }
}

impl fmt::Debug for ExternalInterrupt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ExternalInterrupt").field(&self.nr).finish()
    }
}

impl From<ExternalInterrupt> for u16 {
    fn from(value: ExternalInterrupt) -> Self {
        value.nr
    }
}

//! Nested Vectored Interrupt Controller

use core::arch::asm;
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{self, AtomicBool};

use mmio::SafeRwRegs;

use crate::vtor::{ExternalInterrupt, VectActive};
use crate::{scs, vtor};

const NUM_ISER: usize = (vtor::NUM_INTERRUPTS - 1) / 32 + 1;

/// Nested Vectored Interrupt Controller
pub struct Nvic {
    available: [u32; NUM_ISER],
    num_group_priority_bits: u8,
}

impl Nvic {
    /// Acquires a handle to the NVIC
    ///
    /// # Side effects
    /// - Non-fault exceptions will be globally masks with `CPSID I`
    /// - Masks all external interrupts with `NVIC_ICER`
    ///
    /// # Panics
    /// - if called from any context other than `VectActive::ThreadMode`
    /// - if acquired more than once
    pub fn acquire() -> Self {
        static ONCE: AtomicBool = AtomicBool::new(false);

        assert_eq!(
            VectActive::ThreadMode,
            VectActive::get(),
            "NVIC can only be used from thread mode"
        );

        assert!(
            ONCE.compare_exchange(
                false,
                true,
                atomic::Ordering::Relaxed,
                atomic::Ordering::Relaxed,
            )
            .is_ok(),
            "cannot acquire NVIC more than once"
        );

        Self::acquire_inner()
    }

    /// Returns an iterator over all implemented interrupts
    pub fn interrupts(&self) -> impl Iterator<Item = ExternalInterrupt> + 'static {
        struct Iter {
            available: [u32; NUM_ISER],
            index: u8,
        }

        impl Iterator for Iter {
            type Item = ExternalInterrupt;

            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    let iser = self.available.get_mut(usize::from(self.index))?;
                    if *iser == 0 {
                        self.index += 1;
                        continue;
                    }

                    let offset = iser.trailing_zeros() as u16;
                    *iser &= !(1 << offset);
                    return Some(ExternalInterrupt {
                        nr: 32 * u16::from(self.index) + offset,
                    });
                }
            }
        }

        Iter {
            available: self.available,
            index: 0,
        }
    }

    /// Returns `true` if the given external interrupt is implemented
    pub fn is_implemented(&self, int: ExternalInterrupt) -> bool {
        let index = usize::from(int.nr) / 32;
        let offset = int.nr % 32;

        self.available[index] & (1 << offset) != 0
    }

    /// Returns the number of external interrupts this NVIC supports
    pub fn num_interrupts(&self) -> u16 {
        self.available
            .iter()
            .map(|bits| bits.count_ones() as u16)
            .sum()
    }

    /// Returns the number of priority bits this NVIC supports
    pub fn num_priority_bits(&self) -> u8 {
        self.num_group_priority_bits
    }

    /// Returns the maximum logical priority this NVIC supports
    pub fn max_group_priority(&self) -> u8 {
        1u8.checked_shl(self.num_group_priority_bits.into())
            .map(|shifted| shifted - 1)
            .unwrap_or(u8::MAX)
    }

    /// Enables the specified `int`errupt
    pub fn enable(int: ExternalInterrupt) {
        let index = usize::from(int.nr) / 32;
        let offset = int.nr % 32;

        if let Some(iser) = NVIC_ISER.get(index) {
            iser.write(1 << offset)
        }
    }

    /// Disables the specified `int`errupt
    pub fn disable(int: ExternalInterrupt) {
        let index = usize::from(int.nr) / 32;
        let offset = int.nr % 32;

        if let Some(icer) = NVIC_ICER.get(index) {
            icer.write(1 << offset)
        }
    }

    /// Sets the logical priority for the specified interrupt
    pub fn set_group_priority(&self, int: ExternalInterrupt, logical: u8) {
        assert!(logical <= self.max_group_priority());
        let hw_prio = hw2logical(logical, self.num_group_priority_bits);

        if let Some(ipr) = NVIC_IPR.get(int.nr.into()) {
            ipr.write(hw_prio);
        }
    }

    /// Sets a `handler` for the specified `interrupt` with the given `initial_state`
    pub fn set_handler(&self, interrupt: ExternalInterrupt, handler: extern "C" fn()) {
        interrupt.set_handler(handler);
    }

    /// Sets a stateful `handler` for the specified `interrupt` with the given `initial_state`
    pub fn set_stateful_handler<T>(
        &self,
        interrupt: ExternalInterrupt,
        _handler: T,
        initial_state: T::State,
    ) where
        T: StatefulHandler,
    {
        extern "C" fn handler<T>()
        where
            T: StatefulHandler,
        {
            // SAFETY: interrupts are not reentrant
            T::State::on_interrupt(unsafe { &mut *T::state().get() });
        }

        // SAFETY: interrupts are disabled; this is only called once because `handler` is
        // owned and gets consumed by this method
        unsafe {
            T::state().set(initial_state);
        }
        interrupt.set_handler(handler::<T>);
    }

    /// Globally enables interrupts
    ///
    /// Consumes the `Nvic` so priorities cannot be changed after this point
    pub fn enable_interrupts(self) {
        // SAFETY: operation is memory safe
        unsafe { asm!("CPSIE I") }
    }

    /// Sets `interrupt` as pending
    pub fn set_pending(interrupt: ExternalInterrupt) {
        let nr = interrupt.nr;
        let index = nr / 32;
        let offset = nr % 32;
        if let Some(ispr) = NVIC_ISPR.get(index.into()) {
            ispr.write(1 << offset)
        }
    }

    fn acquire_inner() -> Self {
        // disable external interrupts
        // SAFETY: operation is memory safe
        unsafe { asm!("CPSID I") }

        let mut available = [0; NUM_ISER];
        let mut first = None;
        for (n, (iser, slot)) in NVIC_ISER.iter().zip(&mut available).enumerate() {
            iser.write(!0);
            let mask = iser.read();
            *slot = mask;

            let offset = mask.trailing_zeros();
            if offset != 32 && first.is_none() {
                first = Some(32 * n + offset as usize)
            }
        }

        // undo interrupt enabling
        for icer in NVIC_ICER.iter() {
            icer.write(!0);
        }

        let first = first.expect("0 interrupts available");
        let ipr = NVIC_IPR.get(first).unwrap();
        ipr.write(!0);
        let mask = ipr.read();
        let num_priority_bits = mask.count_ones() as u8;
        ipr.write(0);
        assert!(
            num_priority_bits >= 3,
            "ISA requires at least 3 priority bits but probed {num_priority_bits} bits"
        );

        let aircr = scs::AIRCR.read();
        let prigroup = (aircr >> 8) as u8 & 0b111;
        assert_eq!(0, prigroup);
        let num_group_priority_bits = num_priority_bits.min(7 - prigroup);

        Self {
            available,
            num_group_priority_bits,
        }
    }
}

/// Creates an owned stateful interrupt handler
#[macro_export]
macro_rules! interrupt_handler {
    ($state:ty) => {{
        use core::sync::atomic::{self, AtomicBool};
        static ONCE: AtomicBool = AtomicBool::new(false);
        let ordering = atomic::Ordering::Relaxed;
        assert!(
            ONCE.compare_exchange(false, true, ordering, ordering)
                .is_ok(),
            "cannot acquire a `Handler` more than once"
        );
        struct S {
            inner: (),
        }
        unsafe impl $crate::nvic::StatefulHandler for S {
            type State = $state;
            fn state() -> &'static $crate::nvic::Static<Self::State> {
                static STATE: $crate::nvic::Static<$state> = $crate::nvic::Static::new();
                &STATE
            }
        }
        S { inner: () }
    }};
}

/// # Safety
/// - do not implement this manually; use the `interrupt_handler!` macro
pub unsafe trait StatefulHandler {
    /// The state of this stateful interrupt handler
    type State: HandlerState;
    #[doc(hidden)]
    fn state() -> &'static Static<Self::State>;
}

/// Interrupt handler state
pub trait HandlerState: 'static {
    /// Called when interrupt is serviced
    fn on_interrupt(&mut self);
}

#[doc(hidden)]
pub struct Static<T> {
    inner: UnsafeCell<MaybeUninit<T>>,
}

#[allow(clippy::new_without_default)]
impl<T> Static<T> {
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// # Safety
    /// - must be called with interrupts disabled
    /// - must be called exactly once
    unsafe fn set(&self, initial_value: T) {
        // SAFETY: see `Safety` section on this method's API docs
        unsafe {
            self.inner.get().cast::<T>().write(initial_value);
        }
    }

    fn get(&self) -> *mut T {
        self.inner.get().cast()
    }
}

/// SAFETY: even if downstream code gets a reference to the static it cannot operate on it
unsafe impl<T> Sync for Static<T> where T: Send {}

const NVIC: usize = 0xE000_E000;
// SAFETY: cross checked against TRM
const NVIC_ISER: SafeRwRegs<NVIC, { NUM_ISER }, u32> = unsafe { SafeRwRegs::new(0x100) };
// SAFETY: cross checked against TRM
const NVIC_ICER: SafeRwRegs<NVIC, { NUM_ISER }, u32> = unsafe { SafeRwRegs::new(0x180) };
// SAFETY: cross checked against TRM
const NVIC_ISPR: SafeRwRegs<NVIC, { NUM_ISER }, u32> = unsafe { SafeRwRegs::new(0x200) };
// SAFETY: cross checked against TRM
const NVIC_IPR: SafeRwRegs<NVIC, { vtor::NUM_INTERRUPTS }, u8> = unsafe { SafeRwRegs::new(0x400) };

const fn max_prio(num_prio_bits: u8) -> u8 {
    ((1u16 << num_prio_bits) - 1) as u8
}

const fn hw2logical(logical: u8, num_prio_bits: u8) -> u8 {
    (max_prio(num_prio_bits) - logical) << (8 - num_prio_bits)
}

const _TESTS: () = {
    assert!(224 == hw2logical(0, 3));
    assert!(7 == max_prio(3));
    assert!(0 == hw2logical(7, 3));

    assert!(240 == hw2logical(0, 4));
    assert!(15 == max_prio(4));
    assert!(0 == hw2logical(15, 4));

    assert!(248 == hw2logical(0, 5));
    assert!(31 == max_prio(5));
    assert!(0 == hw2logical(31, 5));

    assert!(252 == hw2logical(0, 6));
    assert!(63 == max_prio(6));
    assert!(0 == hw2logical(63, 6));

    assert!(254 == hw2logical(0, 7));
    assert!(127 == max_prio(7));
    assert!(0 == hw2logical(127, 7));

    assert!(255 == hw2logical(0, 8));
    assert!(255 == max_prio(8));
    assert!(0 == hw2logical(255, 8));
};

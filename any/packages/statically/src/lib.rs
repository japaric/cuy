//! Pointers into statically allocated memory

#![no_std]

use core::ops;
use core::ptr::NonNull;

/// Statically allocated, owning pointer
pub struct Owned<T> {
    inner: NonNull<T>,
}

impl<T> Owned<T> {
    /// # Safety Requirements
    /// - `ptr` must point to a valid memory allocation
    /// - must semantically take ownership of memory allocation behind `ptr`
    #[doc(hidden)]
    pub unsafe fn new(ptr: *mut T) -> Self {
        Self {
            // SAFETY: won't be aliased if Safety Requirements are respected
            inner: unsafe { NonNull::new_unchecked(ptr) },
        }
    }
}

impl<T> ops::Deref for Owned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: this is technically a exclusive reference
        unsafe { self.inner.as_ref() }
    }
}

impl<T> ops::DerefMut for Owned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: this is technically a exclusive reference
        unsafe { self.inner.as_mut() }
    }
}

/// Creates an owning pointer
#[macro_export]
macro_rules! owned {
    ($ty:ty, $expr:expr) => {{
        use core::sync::atomic::{self, AtomicBool};
        static ONCE: AtomicBool = AtomicBool::new(false);
        let ordering = atomic::Ordering::Relaxed;
        assert!(
            ONCE.compare_exchange(false, true, ordering, ordering)
                .is_ok(),
            "cannot acquire `Owned` more than once"
        );
        use core::mem::MaybeUninit;
        static mut OWNED: MaybeUninit<$ty> = MaybeUninit::uninit();
        let ptr = (&raw mut OWNED).cast::<$ty>();
        unsafe {
            ptr.write($expr);
            $crate::Owned::new(ptr)
        }
    }};
}

/// Statically allocated, shared pointer
pub struct Shared<T> {
    inner: NonNull<T>,
}

impl<T> ops::Deref for Shared<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: this is technically a shared reference
        unsafe { self.inner.as_ref() }
    }
}

impl<T> From<Owned<T>> for Shared<T> {
    fn from(owned: Owned<T>) -> Self {
        Self { inner: owned.inner }
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Shared<T> {}

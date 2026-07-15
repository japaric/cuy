#![no_std]

use core::ops;
use core::ptr::NonNull;

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
        unsafe { self.inner.as_ref() }
    }
}

impl<T> ops::DerefMut for Owned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.inner.as_mut() }
    }
}

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
        }

        $crate::Owned::new(ptr)
    }};
}

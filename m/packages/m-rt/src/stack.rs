//! Stack analysis

/// the stack is initialized to this value on boot
pub const COLOR: u32 = 0x90a9_59ff;

/// Returns the maximum stack usage observed so far
pub fn max_usage() -> usize {
    /// # Safety
    /// - `lower..higher` must point into the boundaries of the call stack
    #[optimize::size]
    unsafe extern "C" fn binary_search(lower: *const u32, higher: *const u32, color: u32) -> usize {
        if lower == higher {
            return 0;
        }
        // `lower..higher` is open ended so adjust the end address
        // SAFETY: stack is not zero sized as per previous check so this still points into the stack
        let usable_higher = unsafe { higher.offset(-1) };

        // SAFETY: valid pointer given function's Safety Requirements
        if unsafe { usable_higher.read_volatile() } == color {
            return (higher as usize).wrapping_sub(lower as usize);
        }
        let mut bad = usable_higher as usize;

        // SAFETY: valid pointer given function's Safety Requirements
        if unsafe { lower.read_volatile() } != color {
            return 0;
        }
        let mut good = lower as usize;

        loop {
            // `bad` is always higher than `good` because the stack grows downwards
            let gap = bad.wrapping_sub(good);
            let next = good.wrapping_add(gap / 2) & !0b11;
            if next == good {
                break;
            }
            // SAFETY: aligned as per mask operation; always points into stack memory due to
            // averaging logic
            let value = unsafe { (next as *const u32).read_volatile() };
            if value == color {
                good = next;
            } else {
                bad = next;
            }
        }

        (higher as usize).wrapping_sub(bad)
    }

    unsafe extern "C" {
        static _stack_lower: u32;
        static _stack_higher: u32;
    }

    // SAFETY: these are the boundaries of the call stack
    unsafe { binary_search(&raw const _stack_lower, &raw const _stack_higher, COLOR) }
}

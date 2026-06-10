//! check the the stack is below the static variables to avoid data corruption on overflow
//@ runner: $RUNNER
//@ target: $TARGET

#![no_std]
#![no_main]

use core::sync::atomic::AtomicBool;

use m_rt::LinkerSection;

m_rt::entry!(main);

static IN_BSS: AtomicBool = AtomicBool::new(false);
static IN_DATA: AtomicBool = AtomicBool::new(true);

fn main() -> ! {
    // volatile operations to keep variables in final image
    // SAFETY: read-only access
    unsafe {
        IN_BSS.as_ptr().read_volatile();
        IN_DATA.as_ptr().read_volatile();
    }

    let bss = LinkerSection::bss();
    let data = LinkerSection::data();
    let stack = LinkerSection::stack();
    assert!(stack.higher() <= bss.lower());
    assert!(stack.higher() <= data.lower());

    sh::exit()
}

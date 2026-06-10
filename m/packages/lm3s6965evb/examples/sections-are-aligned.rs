//! check the alignment of linker sections
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

    let vectors = LinkerSection::vectors();
    // REQ004: .vectors is 128-byte aligned
    assert_eq!(0, vectors.lower() % 128);

    _ = LinkerSection::text();
    _ = LinkerSection::rodata();

    // REQ006: .bss is 8-byte aligned
    let bss = LinkerSection::bss();
    assert_eq!(0, bss.lower() % 8);
    assert_eq!(0, bss.higher() % 8);

    let data = LinkerSection::data();
    // REQ007: .data is 8-byte aligned
    assert_eq!(0, data.lower() % 8);
    assert_eq!(0, data.higher() % 8);

    let stack = LinkerSection::stack();
    // REQ005: .stack is 8-byte aligned
    assert_eq!(0, stack.lower() % 8);
    assert_eq!(0, stack.higher() % 8);

    sh::exit()
}

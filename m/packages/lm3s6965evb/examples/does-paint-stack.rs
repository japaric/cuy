//! REQ008 does paint the `.stack` section
//@ runner: $RUNNER
//@ target: $TARGET

#![no_std]
#![no_main]

use m_rt::LinkerSection;

const EXPECTED: u32 = 0x90a9_59ff;

m_rt::entry!(main);

fn main() -> ! {
    let stack = LinkerSection::stack();
    let middle = (((stack.lower() + stack.higher()) / 2) & !0b11) as *const u32;
    // SAFETY: properly aligned and points into valid memory
    let got = unsafe { middle.read_volatile() };
    assert_eq!(EXPECTED, got);

    sh::exit()
}

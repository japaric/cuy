//! REQ008 does paint the `.stack` section
//@ runner: $RUNNER
//@ target: $TARGET

#![no_std]
#![no_main]

use m_rt::{LinkerSection, stack};
use sh::eprintln;

const EXPECTED: u32 = stack::COLOR;

m_rt::entry!(main);

fn main() -> ! {
    let stack = LinkerSection::stack();
    // SAFETY: properly aligned and points into valid memory
    let lower = unsafe { (stack.lower() as *const u32).read_volatile() };
    assert_eq!(EXPECTED, lower);

    let middle = (((stack.lower() + stack.higher()) / 2) & !0b11) as *const u32;
    // SAFETY: properly aligned and points into valid memory
    let middle = unsafe { middle.read_volatile() };
    assert_eq!(EXPECTED, middle);

    // SAFETY: properly aligned and points into valid memory
    let higher = unsafe { (stack.higher() as *const u32).offset(-1).read_volatile() };
    eprintln!("higher={higher:#010x}");
    assert_ne!(EXPECTED, higher);

    sh::exit()
}

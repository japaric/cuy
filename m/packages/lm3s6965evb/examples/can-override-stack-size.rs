//! can override the default stack size
//@ runner: $RUNNER
//@ target: $TARGET

#![no_std]
#![no_main]

use m_rt::LinkerSection;

m_rt::entry!(main, stack_size = 8 * 1024);

fn main() -> ! {
    let stack = LinkerSection::stack();
    assert_eq!(8 * 1024, stack.size());

    sh::exit()
}

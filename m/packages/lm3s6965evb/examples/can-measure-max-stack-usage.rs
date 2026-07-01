//! can measure max stack usage
//@ runner: $RUNNER
//@ target: $TARGET

#![no_std]
#![no_main]

use m_rt::stack;
use sh::eprintln;

m_rt::entry!(main, stack_size = 1024);

fn main() -> ! {
    let max_stack_usage = stack::max_usage();
    eprintln!("max_stack_usage={max_stack_usage}");

    assert_ne!(0, max_stack_usage);
    assert!(max_stack_usage < 1024);

    sh::exit()
}

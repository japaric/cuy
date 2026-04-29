//! check that the configured boot stack size is respected
// runner: qemu-system-aarch64 -cpu cortex-a53 -machine virt -nographic -semihosting -kernel

#![no_std]
#![no_main]

use rt::LinkerSection;

rt::entry!(main, stack_size = 4 * 1024 - 1);

const ALIGN: usize = 64;

fn main() -> ! {
    let section = LinkerSection::stack();
    assert_eq!(
        4 * 1024,
        section.size(),
        "boot stack size does not match configured value"
    );
    assert_eq!(
        0,
        section.lower() % ALIGN,
        "boot stack is not {ALIGN}-byte aligned"
    );
    assert_eq!(
        0,
        section.higher() % ALIGN,
        "boot stack is not {ALIGN}-byte aligned"
    );

    sh::exit()
}

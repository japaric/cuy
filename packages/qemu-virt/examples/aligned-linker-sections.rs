//! REQ002 check that the linker sections are 64-byte aligned
// runner: qemu-system-aarch64 -cpu cortex-a53 -machine virt -nographic -semihosting -kernel

#![no_std]
#![no_main]

use core::sync::atomic::AtomicU8;

use rt::LinkerSection;

rt::entry!(main);

static IN_DATA: AtomicU8 = AtomicU8::new(1);
static IN_BSS: AtomicU8 = AtomicU8::new(0);

const ALIGN: usize = 64;

fn main() -> ! {
    // force some static variables into the final image to make .bss and .data not empty
    // SAFETY: read of valid raw pointer
    unsafe {
        IN_BSS.as_ptr().read_volatile();
        IN_DATA.as_ptr().read_volatile();
    }

    let all_sections = [
        LinkerSection::text(),
        LinkerSection::rodata(),
        LinkerSection::data(),
        LinkerSection::bss(),
        LinkerSection::stack(),
    ];

    for section in &all_sections {
        assert_ne!(0, section.size(), "section must not be empty");
        assert_eq!(
            0,
            section.lower() % ALIGN,
            "lower boundary is not {ALIGN}-byte aligned"
        );
        assert_eq!(
            0,
            section.higher() % ALIGN,
            "higher boundary is not {ALIGN}-byte aligned"
        );
    }

    for (i, section) in all_sections.iter().enumerate() {
        for other in &all_sections[i + 1..] {
            assert_ne!(section, other, "repeated section");
        }
    }

    sh::exit()
}

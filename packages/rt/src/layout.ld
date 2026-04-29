SECTIONS
{
  /* REQQ002 align sections to 64-byte boundaries for MPU compatibility [ARM-R64/D6.1.1] */
  .text ALIGN(64) :
  {
    _text_lower = .;
    *(.text.*);
    _text_higher = ALIGN(64);
  } >ro

  .rodata ALIGN(64) :
  {
    _rodata_lower = .;
    *(.rodata.*);
    _rodata_higher = ALIGN(64);
  } >ro

  .data ALIGN(64) :
  {
    _data_lower = .;
    *(.data.*);
    _data_higher = ALIGN(64);
  } >rw

  /* REQ000 */
  .bss (NOLOAD) : ALIGN(64)
  {
    _bss_lower = .;
    *(.bss.*);
    _bss_higher = ALIGN(64);
  } >rw

  .stack (NOLOAD) : ALIGN(64)
  {
    _stack_lower = .;
    KEEP(*(.stack.boot));
    . = ALIGN(64);
    _boot_stack_higher = .;
    *(.stack);
    _stack_higher = .;
  } >rw

  /* discard non-debug sections unless there's a clear reason to */
  /* add them to the image */
  /DISCARD/ :
  {
    *(.eh_frame);
    *(.eh_frame_hdr);
    *(.got);
  }
}

ASSERT(
    _boot_stack_higher % 16 == 0,
    "boot stack (initial SP value) is not 16-byte aligned as required by ISA");

SECTIONS
{
  .text :
  {
    *(.text.*);
  } >ro

  .rodata :
  {
    *(.rodata.*);
  } >ro

  .data :
  {
    *(.data.*);
  } >rw

  /* REQ000 */
  /* aligned to 8-byte to optimize zero-ing loop */
  .bss (NOLOAD) : ALIGN(8)
  {
    _bss_start = .;
    *(.bss.*);
    . = ALIGN(8);
    _bss_end = .;
  } >rw

  /* the MPU usually operates on 4 KiB pages [citation needed] even if it can */
  /* support granularity as fine as 64B [ARM-R64/D6.1.1] */
  .stack (NOLOAD) : ALIGN(4K)
  {
    /* TODO maybe make configurable; start with 4 KiB for the boot core */
    . += 4K;
    _boot_stack_end = .;

    *(.stack.*);
    . = ALIGN(4K);
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

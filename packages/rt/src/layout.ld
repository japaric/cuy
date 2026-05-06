/* (NOLOAD) marks the *section* header as NOBITS but by default the section is */
/* still mapped to a LOAD *program* header which the FVP will */
/* zero-initialize instead of fill with a configurable byte pattern */
/* the override the default section header to program header mapping we use */
/* the custom program headers defined here */
PHDRS
{
  load PT_LOAD;
  noload PT_NULL;
}

SECTIONS
{
  /* REQQ002 align sections to 64-byte boundaries for MPU compatibility [ARM-R64/D6.1.1] */
  .text ALIGN(64) :
  {
    _text_lower = .;
    *(.text.*);
    /* NOTE: this does NOT align the section to a 64-byte boundary as reported by `size` */
    /* _text_higher = ALIGN(64); */
    /* however, this does */
    . = ALIGN(64);
    /* this simply creates a binding; it has no effect on the section itself */
    _text_higher = .;
  } >ro :load

  .rodata ALIGN(64) :
  {
    _rodata_lower = .;
    *(.rodata.*);
    . = ALIGN(64);
    _rodata_higher = .;
  } >ro :load

  .data ALIGN(64) :
  {
    _data_lower = .;
    *(.data.*);
    . = ALIGN(64);
    _data_higher = .;
  } >rw :load

  /* REQ000 */
  .bss (NOLOAD) : ALIGN(64)
  {
    _bss_lower = .;
    *(.bss.*);
    . = ALIGN(64);
    _bss_higher = .;
  } >rw :noload

  .stack (NOLOAD) : ALIGN(64)
  {
    _stack_lower = .;
    KEEP(*(.stack.boot));
    . = ALIGN(64);
    _boot_stack_higher = .;

    *(.stack);
    _stack_higher = .;
  } >rw :noload

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
    _bss_lower % 8 == 0 && _bss_higher % 8 == 0 ,
    ".bss section is not 8-byte aligned as assumed by startup code");

ASSERT(
    _boot_stack_higher % 16 == 0,
    "boot stack (initial SP value) is not 16-byte aligned as required by ISA");

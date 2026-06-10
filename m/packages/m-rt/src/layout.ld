/* default entry point for LLD but make it explicit */
ENTRY(_start);

SECTIONS
{
  .vectors :
  {
    _stack_higher = ORIGIN(rw) + LENGTH(rw);

    _vectors_lower = .;

    LONG(_stack_higher);
    LONG(_start | 1); /* set thumb bit */

    _vectors_higher = .;
  } > ro

  .text :
  {
    _text_lower = .;

    *(.text.*);

    _text_higher = .;
  } > ro

  .rodata :
  {
    _rodata_lower = .;

    *(.rodata.*);

    /* align the end of this section to 8-byte boundary so */
    /* _data_lma is also 8-byte aligned */
    . = ALIGN(8);
    _rodata_higher = .;
  } > ro

  _data_lma = _rodata_higher;
  .data : AT(_data_lma)
  {
    . = ALIGN(8);
    _data_lower = .;

    *(.data.*);

    . = ALIGN(8);
    _data_higher = .;
  } > rw

  .bss (NOLOAD) :
  {
    . = ALIGN(8);
    _bss_lower = .;

    *(.bss.*);

    . = ALIGN(8);
    _bss_higher = .;
    _stack_lower = .;
  } > rw :NONE

  /* discard non-debug sections unless there's a clear reason to */
  /* add them to the image */
  /DISCARD/ :
  {
    *(.got);
    *(.ARM.exidx);
  }
}

/* REQ004: .vectors is 128-byte aligned */
ASSERT(0 == ADDR(.vectors) % 128, "vector table must at least be 128-byte aligned");

/* REQ005: .stack is 8-byte aligned */
ASSERT(0 == _stack_higher % 8 && 0 == _stack_lower % 8, "stack is not 8-byte aligned");

/* REQ006: .bss is 8-byte aligned */
ASSERT(0 == _bss_lower % 8 && 0 == _bss_higher % 8, ".bss boundaries are not 8-byte aligned");

/* REQ007: .data is 8-byte aligned */
ASSERT(0 == _data_lma % 8, ".data LMA is not 8-byte aligned");
ASSERT(0 == _data_lower % 8 && 0 == _data_higher % 8, ".data VMA boundaries are not 8-byte aligned");

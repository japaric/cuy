/* default entry point for LLD but make it explicit */
ENTRY(_start);

SECTIONS
{
  .vectors :
  {
    _stack_higher = ORIGIN(rw) + LENGTH(rw);
    LONG(_stack_higher);
    LONG(_start | 1); /* set thumb bit */
  } > ro

  .text :
  {
    *(.text.*);
  } > ro

  .rodata :
  {
    *(.rodata.*);
  } > ro

  .data :
  {
    *(.data.*);
  } > rw

  .bss : ALIGN(8)
  {
    _bss_lower = .;
    *(.bss.*);
    . = ALIGN(8);
    _bss_higher = .;
    _stack_lower = .;
  } > rw

  /* discard non-debug sections unless there's a clear reason to */
  /* add them to the image */
  /DISCARD/ :
  {
    *(.got);
    *(.ARM.exidx);
  }
}

ASSERT(0 == ADDR(.vectors) % 128, "vector table must at least be 128-byte aligned");
ASSERT(0 == _stack_higher % 8 && 0 == _stack_lower % 8, "stack is not 8-byte aligned");

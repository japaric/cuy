  /* NOTE do not add code unless it fixes a failing test */
  .section .text._start
  .global _start
_start:

  /* REQ000: zero bss */
  /* NOTE: this assumes the section is 8-byte aligned */
  ldr r0, =_bss_lower
  ldr r1, =_bss_higher
  mov r2, #0x0
1:
  cmp  r0, r1
  bhs  2f
  strd r2, r2, [r0], #8
  b    1b

2:
  /* jump into the Rust part of the entry point */
  b rust_start

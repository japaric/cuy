  /* NOTE do not add code unless it fixes a failing test */
  .section .text._start
  .global _start
_start:

  /* REQ000: zero bss */
  /* NOTE: this assumes the section is 8-byte aligned (REQ006) */
  ldr r0, =_bss_lower
  ldr r1, =_bss_higher
  mov r2, #0x0
1:
  cmp  r0, r1
  bhs  2f
  strd r2, r2, [r0], #8
  b    1b

2:
  /* REQ003: initialize the .data section */
  /* NOTE: this assumes that the section LMA and VMA are 8-byte aligned (REQ007) */
  ldr  r0, =_data_lma
  ldr  r1, =_data_lower
  ldr  r2, =_data_higher
3:
  cmp  r1, r2
  bhs  4f
  ldrd r3, r4, [r0], #8
  strd r3, r4, [r1], #8
  b    3b

4:
  /* jump into the Rust part of the entry point */
  b rust_start

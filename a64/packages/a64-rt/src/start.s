  /* NOTE do not add code unless it fixes a failing test */
  .section .text._start
  .global _start
_start:
  /* trap non-boot cores otherwise we get data races (unsound) in startup code */
  mov  x1, 0xffffffffff
  mrs  x0, MPIDR_EL1
  movk x1, 0xff, lsl #16
  /* if MPIDR_EL1 & AFFINITY_MASK (0xff_00ff_ffff) == 0 */
  tst  x0, x1
  b.eq is_boot_core

  /* TODO need an API to take these cores out of this sleep loop */
is_secondary_core:
  wfi
  b is_secondary_core

is_boot_core:
  /* set up the stack pointer; without stack space nothing works */
  ldr x0, =_boot_stack_higher
  mov SP, x0

  /* REQ001: allow FPU/SIMD in EL1; codecov uses these kind of registers */
  mrs x0, CPACR_EL1
  add x0, x0, #(3 << 20)
  msr CPACR_EL1, x0

  /* REQ000: zero bss */
  /* NOTE: this assumes the section is 8-byte aligned */
  ldr x0, =_bss_lower
  ldr x1, =_bss_higher
1:
  cmp  x0, x1
  b.hs 2f
  str  xzr, [x0], #0x8
  b    1b

2:
  /* jump into the Rust part of the entry point */
  b rust_start

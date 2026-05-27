/* https://github.com/qemu/qemu/blob/f5a2438/hw/arm/stellaris.c */
MEMORY
{
  FLASH : ORIGIN = 0x00000000, LENGTH = 256K
  SRAM  : ORIGIN = 0x20000000, LENGTH = 64K
}

REGION_ALIAS("ro", FLASH);
REGION_ALIAS("rw", SRAM);

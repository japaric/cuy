/* https://github.com/qemu/qemu/blob/f5a2438/hw/arm/mps2.c */
MEMORY
{
  PSRAM  : ORIGIN = 0x21000000, LENGTH = 16M
  SSRAM1 : ORIGIN = 0x00000000, LENGTH = 4M
}

REGION_ALIAS("ro", SSRAM1);
REGION_ALIAS("rw", PSRAM);

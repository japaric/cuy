/* https://github.com/qemu/qemu/blob/f5a2438/hw/arm/musca.c */
MEMORY
{
  ROM_S  : ORIGIN = 0x10200000, LENGTH = 8M /* matches init_svtor */
  ROM_NS : ORIGIN = 0x00200000, LENGTH = 8M
  SRAM   : ORIGIN = 0x00000000, LENGTH = 2M
}

REGION_ALIAS("ro", ROM_S);
REGION_ALIAS("rw", SRAM);

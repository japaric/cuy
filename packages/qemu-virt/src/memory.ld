MEMORY
{
  /* https://github.com/qemu/qemu/blob/master/hw/arm/virt.c */
  FLASH : ORIGIN = 0x00000000, LENGTH = 128M
  /* TODO is 128 MiB the default RAM size? */
  RAM   : ORIGIN = 0x40000000, LENGTH = 128M
}

/* FIXME QEMU complains when FLASH is aliased to 'ro' */
REGION_ALIAS("ro", RAM);
REGION_ALIAS("rw", RAM);

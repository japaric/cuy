MEMORY
{
  /* DRAM is mapped to address 0 */
  /* reference: https://developer.arm.com/documentation/100964/1128/Base-Platform/Base-Platform-memory/BaseR-Platform-memory-map/ */
  /* *
  /* `--list-memory` reports many overlapping memory regions of different */
  /* sizes starting at address 0 and it's unclear if the FVP raises an */
  /* exception when accessing unimplemented memory */
  /* secureDRAM is 32 MiB whereas other '*_dram' regions exceed the 2 GiB mark */
  /* we'll use the smaller value */
  DRAM : ORIGIN = 0, LENGTH = 32M
}

REGION_ALIAS("ro", DRAM);
REGION_ALIAS("rw", DRAM);

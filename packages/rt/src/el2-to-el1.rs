    ldr x8, ={trampoline}
    msr ELR_EL2, x8
    mrs x8, DAIF
    mov w9, #0x5
    orr x8, x8, x9
    msr SPSR_EL2, x8
    eret

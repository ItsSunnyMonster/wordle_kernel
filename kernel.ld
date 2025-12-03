ENTRY(_start)

SECTIONS {
    . = 1M; /* Avoid overwriting bios or bootloader memory under 1MiB */

    .text : ALIGN(8)
    {
        KEEP(*(.multiboot_header))
        *(.text*)
    }

    .rodata :
    { 
        *(.rodata*) 
    }

    .data :
    {
        *(.data*)
    }

    .bss :
    {
        *(COMMON)
        *(.bss*)
    }

    /DISCARD/ :
    {
        *(.eh_frame*)
        *(.comment)
    }

}
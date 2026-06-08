ENTRY(_start)

KERNEL_VMA = 0xffffffff80000000;

SECTIONS
{
    . = KERNEL_VMA;

    . = ALIGN(4K);
    .text : ALIGN(4K)
    {
        *(.text .text.*)
    }

    . = ALIGN(4K);
    .rodata : ALIGN(4K)
    {
        *(.rodata .rodata.*)
    }

    . = ALIGN(4K);
    .got : ALIGN(4K)
    {
        *(.got .got.*)
    }

    . = ALIGN(4K);
    .data : ALIGN(4K)
    {
        *(.data .data.*)
    }

    . = ALIGN(4K);
    .bss : ALIGN(4K)
    {
        *(COMMON)
        *(.bss .bss.*)
    }

    /DISCARD/ :
    {
        *(.comment)
        *(.eh_frame .eh_frame_hdr)
        *(.note .note.*)
    }
}

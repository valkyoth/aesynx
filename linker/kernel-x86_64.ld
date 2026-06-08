ENTRY(_start)

KERNEL_VMA = 0xffffffff80000000;

PHDRS
{
    text PT_LOAD FLAGS(5);
    rodata PT_LOAD FLAGS(4);
    data PT_LOAD FLAGS(6);
}

SECTIONS
{
    . = KERNEL_VMA;

    . = ALIGN(4K);
    .text : ALIGN(4K)
    {
        *(.text .text.*)
    } :text

    . = ALIGN(4K);
    .rodata : ALIGN(4K)
    {
        *(.rodata .rodata.*)
        *(.got .got.*)
    } :rodata

    . = ALIGN(4K);
    .data : ALIGN(4K)
    {
        . = ALIGN(8);
        KEEP(*(.limine_requests_start))
        KEEP(*(.limine_requests))
        KEEP(*(.limine_requests_end))
        . = ALIGN(8);
        *(.data .data.*)
    } :data

    . = ALIGN(4K);
    .bss : ALIGN(4K)
    {
        *(COMMON)
        *(.bss .bss.*)
    } :data

    __kernel_end = .;

    /DISCARD/ :
    {
        *(.comment)
        *(.eh_frame .eh_frame_hdr)
        *(.note .note.*)
    }
}

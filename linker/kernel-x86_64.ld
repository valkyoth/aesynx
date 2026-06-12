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
        __kernel_text_start = .;
        *(.text .text.*)
        . = ALIGN(4K);
        __kernel_text_end = .;
    } :text

    . = ALIGN(4K);
    .rodata : ALIGN(4K)
    {
        __kernel_rodata_start = .;
        *(.rodata .rodata.*)
        *(.got .got.*)
        . = ALIGN(4K);
        __kernel_rodata_end = .;
    } :rodata

    . = ALIGN(4K);
    __kernel_data_start = .;
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

    . = ALIGN(4K);
    __kernel_data_end = .;

    . = ALIGN(4K);
    __kernel_activation_stack_guard_start = .;
    . += 4K;
    __kernel_activation_stack_guard_end = .;

    . = ALIGN(4K);
    .aesynx_activation_stack (NOLOAD) : ALIGN(4K)
    {
        __kernel_activation_stack_start = .;
        KEEP(*(.aesynx_activation_stack))
        . = ALIGN(4K);
        __kernel_activation_stack_end = .;
    } :data

    . = ALIGN(4K);
    __kernel_end = .;

    /DISCARD/ :
    {
        *(.comment)
        *(.eh_frame .eh_frame_hdr)
        *(.note .note.*)
    }
}

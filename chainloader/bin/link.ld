ENTRY(start)

/* Program headers. Also called segments. */
PHDRS
{
    /*
     * PT_LOAD FLAGS(x):
     * The flags of an ELF program header/segment. Always 32 bit long, even for
     * 64-bit ELFs. Also called "Segment Permissions" in ELF specification or
     * "p_flags". Helps loaders of ELF files to set the right page table bits.
     *
     * Only needed in case that a Multiboot2 boot loader such as GRUB is used
     * on an UEFI system. In that case, paging is already active if we land in
     * the AMD64 EFI machine state.
     */

    loader     PT_LOAD FLAGS(7); /* 0b111 - read + write + execute */
}

SECTIONS {

    /* Binary is relocatable. Note: We want a 2 MiB alignment for huge pages. */
    .loader 2M : AT(2M) ALIGN(4K)
    {
        KEEP(*(.multiboot2_header));

        *(.text .text.*)

        *(.rodata .rodata.*)

        /* COMMON symbols are also part of .bss. */
        *(COMMON)
        *(.bss .bss.*)

        *(.data .data.*)
    } : loader

    /DISCARD/ :
    {
        *(.note.*)
        *(.comment .comment.*)
        *(.eh_frame*)
        *(.got .got.*)
    }

}

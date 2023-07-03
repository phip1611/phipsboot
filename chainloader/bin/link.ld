ENTRY(start)

/*
 * FORMAT: It's hard to find the right names for this. Keyword: bfdname.
 * Also see GNU ld's source.
 */
OUTPUT_FORMAT("elf32-i386")
OUTPUT_ARCH("i386")

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

    loader     PT_LOAD      FLAGS(7); /* 0b111 - +read +write +execute */

    /* dynamic    PT_DYNAMIC   FLAGS(6); *//* 0b110 - +read +write -execute */
}

SECTIONS {

    /* Binary is relocatable. Note: We want a 2 MiB alignment for huge pages. */
    .mb2_hdr 2M : AT(2M) ALIGN(4K)
    {
        KEEP(*(.mb2_hdr));
    } : loader

    /*
     * Linker is friendly enough to automatically increase link and load address
     * according to the size of the previous section. So no need to specify it
     * for every output section. This is true for GNU ld and lld (LLVM).
     */


    .loader :
    {
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

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
     * FLAGS:
     * - 0b111 - +read +write +execute
     * - Strictly speaking irrelevant in legacy boot.
     */
    boot  PT_LOAD FLAGS(7);
    loader PT_LOAD FLAGS(7);
}

LINK_ADDR_BOOT = 2M;
LOAD_ADDR_BOOT = LINK_ADDR_BOOT;

LINK_ADDR_LOADER = 1024M + SIZEOF(.boot);
LOAD_ADDR_LOADER = LOADADDR(.boot) + SIZEOF(.boot);

SECTIONS {

    /*
     * All symbols from the assembly code.
     */
    .boot LINK_ADDR_BOOT : AT(LOAD_ADDR_BOOT) {
        *(.boot .boot.*)
        KEEP(*(.mb2_hdr));
    } : boot

    /*
     * All symbols from the high-level Rust code.
     */
    .loader LINK_ADDR_LOADER : AT(LOAD_ADDR_LOADER)
    {
        *(.text .text.*)

        *(.rodata .rodata.*)

        /* COMMON symbols are also part of .bss. */
        *(COMMON)
        *(.bss .bss.*)

        *(.data .data.*)
    } : loader

    /*
     * General assertions:
     * - Make sure that one single 2 MiB mapping covers the whole loader.
     */
    ASSERT(SIZEOF(.loader) < 2M, "The loader must be less than 2 MiB in size in memory")

    /DISCARD/ :
    {
        *(.comment .comment.*)
        *(.dynamic)
        *(.eh_frame*)
        *(.got .got.*)
        *(.note.*)
        *(.plt .plt.*)
        *(.rela .rela.*)
    }

}

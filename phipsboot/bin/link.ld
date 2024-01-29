/* Symbol from assembly boot code. */
ENTRY(entry_common)

/*
 * FORMAT/ARCH: It's hard to find the right names for this. Keyword: "bfdname".
 * Also see GNU ld's source.
 */
OUTPUT_FORMAT("elf64-x86-64")
OUTPUT_ARCH("i386:x86-64")

/* Program headers. Also called segments. */
PHDRS
{
    /*
     * FLAGS:
     * - 0b111 - +read +write +execute
     * - Strictly speaking irrelevant for our dispatch in 32-bit protected mode,
     *   but technically the right permissions for those segments.
     */
    boot   PT_LOAD FLAGS(7);
    rx     PT_LOAD FLAGS(5);
    ro     PT_LOAD FLAGS(4);
    rw     PT_LOAD FLAGS(6);
    note   PT_NOTE;
}

LINK_ADDR_BOOT = 0x400000; /* 4 MiB */
/*
 * We set these to be equal as everything else just creates confusion.
 * With Multiboot 1 and Xen PVH, we will be loaded at that address, as
 * bootloaders such as GRUB use the load address of an segment when paging is
 * not active.
 *
 * With Multiboot 2, the bootloader might relocate us due to the relocatable
 * tag.
 */
LOAD_ADDR_BOOT = LINK_ADDR_BOOT;

/*
 * Base address for high-level code linkage.
 */
LINK_ADDR_HIGH_BASE = 0xffffffff88200000;

/*
 * Ensure that all LOAD segments are continuous in memory and that they have the
 * same link and load address offset from the base address of the first LOAD
 * segment.
 *
 * This way, we can
 * - properly create page-tables (alignment of virt and phys address must match)
 * - calculate the position of all symbols in physical memory from their
 *   virtual/link address
 */
LINK_ADDR_RX = ALIGN(LINK_ADDR_HIGH_BASE + SIZEOF(.boot), 4K);
LOAD_ADDR_RX = ALIGN(LOAD_ADDR_BOOT + SIZEOF(.boot), 4K);

LINK_ADDR_RO = ALIGN(LINK_ADDR_RX + SIZEOF(.text), 4K);
LOAD_ADDR_RO = ALIGN(LOAD_ADDR_RX + SIZEOF(.text), 4K);

LINK_ADDR_RW = ALIGN(LINK_ADDR_RO + SIZEOF(.rodata), 4K);
LOAD_ADDR_RW = ALIGN(LOAD_ADDR_RO + SIZEOF(.rodata), 4K);

/*
 * Calculate relevant meta data for the page table mappings. These data is
 * used by the assembly boot code.
 */
COUNT_PAGES_RX = (SIZEOF(.text) + 4K - 1) / 4K;
COUNT_PAGES_RO = (SIZEOF(.rodata) + 4K - 1) / 4K;
COUNT_PAGES_RW = ((SIZEOF(.bss) + SIZEOF(.data)) + 4K - 1) / 4K;

SECTIONS {

    /*
     * All symbols from the assembly code.
     */
    .boot LINK_ADDR_BOOT : AT(LOAD_ADDR_BOOT) ALIGN(4k) {
        *(.boot.text)
        /* Push the header a little from the top to simplify `objdump`. */
        KEEP(*(.mb2_hdr));
        *(.boot.data)
    } : boot

    /*
     * All symbols from the high-level Rust code.
     */
    .text LINK_ADDR_RX : AT(LOAD_ADDR_RX)
    {
        *(.text .text.*)
    } : rx

    .rodata LINK_ADDR_RO : AT(LOAD_ADDR_RO)
    {
        *(.rodata .rodata.*)
    } : ro

    .bss LINK_ADDR_RW : AT(LOAD_ADDR_RW)
    {
        *(COMMON)
        *(.bss .bss.*)
    } : rw

    .data : /* Link + Load Addr auto increment */
    {
        *(.data .data.*)
    } : rw

    BIN_SIZE = . - LINK_ADDR_HIGH_BASE;

    .note : {
        *(.note .note.*)
    } : note

    /*
     * General Assertions
     *
     * - Make sure the loader fits into 2 MiB. This ensures that we only need
     *   at most all 512 entries of the L1 page table to map the whole binary.
     * - Ensure that we can use a 2 MiB huge-page mapping during the boot to
     *   access all kind of data from the boot code.
     */
    ASSERT(BIN_SIZE <= 2M, "The binary must be <= 2 MiB in size")
    ASSERT(ALIGN(LINK_ADDR_BOOT, 2M) == LINK_ADDR_BOOT, "LINK_ADDR_BOOT must be 2 MiB aligned")
    ASSERT(ALIGN(LINK_ADDR_HIGH_BASE, 2M) == LINK_ADDR_HIGH_BASE, "LINK_ADDR_HIGH_BASE must be 2 MiB aligned")

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

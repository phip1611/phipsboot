# The assembly file uses GNU Assembly (GAS) language with AT&T syntax.
#
# All macros provided here operate on 32-bit registers and are meant to be used
# to prepare the transition to 64-bit mode.


/*
 * Calculates the real load address from the link address of a known symbol and
 * stores it in the provided register. It relies on the relocation offset already
 * was already set in %ebp.
 *
 * This only works for symbols of the boot code, where link = load address.
 *
 * Clobbers: rflags, reg_out
 */
.macro  M_RESOLVE_RUNTIME_ADDR  link_addr reg_out
    mov  $\link_addr,  \reg_out
    add  %ebp,         \reg_out
.endm


/*
 * Computes the page table offset for a virtual address with x86_64 4-level
 * paging. The function returns the byte offset and not the number of the entry
 * (nr of entry = offset / 4).
 *
 * The upper 32-bits of the virtual address must be provided in %eax and the
 * lower 32-bits in %ebx. The offset will then be stored in %eax.
 *
 * Clobbers: rflags, %eax
 */
.macro  M_CALC_PT_ENTRY_OFFSET  level
    # Compile-time checks for the level.
    .if \level < 1
        .error "Level must be 1, 2, 3, or 4."
    .endif
    .if \level > 4
        .error "Level must be 1, 2, 3, or 4."
    .endif

    # Select the 9 index bits of the current level.
    .if \level <= 2
        shr  $(9 * (\level - 1) + 12), %ebx
        and  $0x1ff, %ebx
        mov  %ebx,   %eax
    .endif
    .if \level == 3
        # Select lowest seven bits from 63..32.
        and  $0x7f, %eax
        shl  $2,    %eax
        # Select the two highest bits from 31..0.
        shr  $30,   %ebx
        and  $3,    %ebx
        or  %ebx,   %eax
    .endif
    .if \level == 4
        # Select bits 15..7 of high address.
        shr  $7,     %eax
        and  $0x1ff, %eax
    .endif

    # Offset = index * size per entry
    imul  $8, %eax
.endm


/**
 * Maps a LOAD segment with information stored behind `mapping_info_ptr` into
 * memory. If the level is one, it does so in a loop until all pages are mapepd.
 *
 * This code relies on the fact that the whole LOAD segment can be loaded into
 * a single PT, i.e., that the highest index is 512. Or in other words, the LOAD
 * segment must not cross a 2 MiB address boundary.
 *
 * Clobbers: rflags, %eax, %ebx, %ecx, %edx, %esp
 */
.macro M_MAP_HIGH_ADDR level, mapping_info_ptr, phys_table, phys_dest, flags
    M_RESOLVE_RUNTIME_ADDR  \mapping_info_ptr %esp
    mov  4(%esp), %eax  # vaddr (bits 63..32)
    mov   (%esp), %ebx  # vaddr (bits 31..0)

    mov  %ebx,  %ecx
    M_CALC_PT_ENTRY_OFFSET \level
    mov  %ecx,  %ebx

    # now: %eax holds the PT offset
    #      %ebx holds addr bits 31..0

    .if \level > 1
        M_RESOLVE_RUNTIME_ADDR \phys_table %ebx

        # entry in %edx
        M_RESOLVE_RUNTIME_ADDR \phys_dest %edx
        or  $(\flags & 0xfff), %edx

        M_WRITE_PT_ENTRY %edx %ebx %eax \flags
    .endif
    .if \level == 1
        mov  8(%esp), %esp  # num of pages

        # %eax now contains the PT offset
        # %ebx contains addr bits 31..0

        # Calculate phys addr of the current load segment:
        # 1) First, we get the runtime addr of the base addr (LOAD_ADDR_BOOT)
        # 2) We then apply the same offset to the phys addr that the virt
        #    segment addr has from its base
        M_RESOLVE_RUNTIME_ADDR \phys_dest %edx
        mov  %ebx, %ecx
        and  $(0x1ff000), %ebx
        add  %ebx,  %edx
        # now: %ebx and %edx have the same offset from the 2 MiB-aligned base

        0:
            cmp  $0, %esp
            je   1f

            M_RESOLVE_RUNTIME_ADDR \phys_table %ecx

            M_WRITE_PT_ENTRY %edx %ecx %eax \flags

            dec   %esp         # decrement counter
            add   $8,    %eax  # increase offset into PT
            add   $4096, %ebx  # next page; add to addr bits 31..0
            jo    2f # Add above assumes that overflows will never happen!
            add   $4096, %edx  # increase phys mem offset from base
            jmp   0b
        2:  ud2
        1:
    .endif

.endm


/**
 * Takes a 32-bit symbol, calculates its actual runtime address, and maps the
 * virtual address for the given level and the given flags.
 *
 * # Parameters
 * - level:      1, 2, 3, or 4
 * - vaddr:      32-bit symbol
 * - phys_table: 32-bit symbol
 * - phys_dest:  32-bit symbol
 * - flags:      12-bit immediate describing the flags (entry bits 11..0)
 *
 * Clobbers: rflags, %eax, %ebx, %edx
 */
.macro M_MAP_SYMBOL32  level, vaddr, phys_table, phys_dest, flags
    M_RESOLVE_RUNTIME_ADDR \vaddr %ebx
    xor  %eax, %eax # zero upper 32-bits
    M_CALC_PT_ENTRY_OFFSET  \level

    # now: %eax holds the PT offset, %ebx can be used again

    M_RESOLVE_RUNTIME_ADDR \phys_table %ebx

    # entry in %edx
    M_RESOLVE_RUNTIME_ADDR \phys_dest %edx

    M_WRITE_PT_ENTRY %edx %ebx %eax \flags
.endm


/**
 * Writes a single page table entry with all specified flags.
 *
 * Clobbers: rflags, reg_phys_table
 */
.macro M_WRITE_PT_ENTRY  reg_entry_31_0, reg_phys_table, reg_offset, flags
    add  \reg_offset,       \reg_phys_table
    or  $(\flags & 0xfff),  \reg_entry_31_0
    mov  \reg_entry_31_0,   (\reg_phys_table)

    # Apply Execute Disable (XD) flag
    movl  $((\flags >> 32) & (1 << 31)), 4(\reg_phys_table)
.endm


/*
 * Adds the given bits to the given control register.
 *
 * Clobbers: rflags, %eax, reg_cr
 */
.macro  M_SET_CR_BITS  reg_cr, bits
    mov  \reg_cr, %eax
    or   $\bits,  %eax
    mov  %eax,    \reg_cr
.endm


/*
 * Adds the given bits to the given MSR. Only uses the lower 32-bits.
 *
 * Clobbers: rflags, %eax, %ecx, %edx, msr
 */
.macro  M_SET_MSR_BITS  msr, bits
    mov  $\msr, %ecx
    # Reads the lower 32-bits of MSR behind %ecx into %eax. The higher
    # 32-bits are read into %edx.
    rdmsr
    or   $\bits, %eax
    wrmsr
.endm

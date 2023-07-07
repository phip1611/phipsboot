# The assembly file uses GNU Assembly (GAS) language with AT&T syntax.

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
 * The virtual address must be provided in %eax. The offset will then be stored
 * in %eax.
 *
 * For simplicity, this function only operates with 32-bit addresses. Hence,
 * bits 63..32 are assumed to be all zero.
 *
 * Clobbers: rflags, %eax
*/
.macro  _M_CALC_PT_ENTRY_OFFSET  level
    # Compile-time checks for the level.
    .if \level < 1
        .error "Level must be 1, 2, 3, or 4."
    .endif
    .if \level > 4
        .error "Level must be 1, 2, 3, or 4."
    .endif

    # Select bits of current level.
    .if \level <= 2
        shr  $(9 * (\level - 1) + 12), %eax
        and  $0x1ff, %eax
    .endif
    .if \level == 3
        # Select the two highest bits.
        shr  $30,  %eax
        and  $0x3, %eax
    .endif
    .if \level == 4
        # As specified by macro description: bits 63..32 are all supposed to be zero.
        mov  $0, %eax
    .endif

    # Offset = index * size per entry
    imul  $8, %eax
.endm


/*
 * Calculates the offset into the page table for the given address.
 *
 * Clobbers: rflags, %eax
 */
.macro  M_CALC_PT_ENTRY_OFFSET  level, vaddr
    mov  $\vaddr, %eax
    _M_CALC_PT_ENTRY_OFFSET  \level
.endm


/*
 * Calculates the offset into the page table for the given address but applies
 * the relocation offset to the address beforehand.
 *
 * Clobbers: rflags, %eax
 */
.macro  M_CALC_PT_ENTRY_OFFSET_RELOC  level, vaddr
    M_RESOLVE_RUNTIME_ADDR   \vaddr, %eax
    _M_CALC_PT_ENTRY_OFFSET  \level
.endm


/*
 * Writes a page-table entry. The offset to the entry into the page table
 * must be provided in %eax.
 *
 * Clobbers: rflags, %eax, %ebx
 */
.macro  _M_CREATE_PT_ENTRY  curr_phys, dest_phys, entry_flags
    # Address of current page tabel entry = base addr + offset
    M_RESOLVE_RUNTIME_ADDR  \curr_phys, %ebx
    add  %eax, %ebx # add entry offset

    # Page table entry = target phys address + flags
    M_RESOLVE_RUNTIME_ADDR  \dest_phys, %eax
    or  $(\entry_flags & 0xfff), %eax

    # Write lower 32-bit of 64-bit page table entry
    mov  %eax, (%ebx)

    # Write upper half: stays zero (just to be sure).
    movl  $0, +4(%ebx)
.endm


/*
 * Create a page-table entry for the given address.
 *
 * Clobbers: rflags, %eax, %ebx
 */
.macro  M_CREATE_PT_ENTRY  vaddr, level, curr_phys, dest_phys, entry_flags
    M_CALC_PT_ENTRY_OFFSET  \level \vaddr
    _M_CREATE_PT_ENTRY  \curr_phys, \dest_phys, \entry_flags
.endm


/*
 * Create a page-table entry for the given address but applies the relocation
 * offset to the address beforehand.
 *
 * Clobbers: rflags, %eax, %ebx
 */
.macro  M_CREATE_PT_ENTRY_RELOC  vaddr, level, curr_phys, dest_phys, entry_flags
    M_CALC_PT_ENTRY_OFFSET_RELOC  \level \vaddr
    _M_CREATE_PT_ENTRY  \curr_phys, \dest_phys, \entry_flags
.endm


/*
 * Adds the given bits to the given control register.
 *
 * Clobbers: %eax, reg_cr
 */
.macro  M_SET_CR_BITS  reg_cr, bits
    mov  \reg_cr, %eax
    or   $\bits,  %eax
    mov  %eax,    \reg_cr
.endm


/*
 * Adds the given bits to the given MSR. Only uses the lower 32-bits.
 *
 * Clobbers: %eax, %ecx, %edx, msr
 */
.macro  M_SET_MSR_BITS  msr, bits
    mov  $\msr, %ecx
    # Reads the lower 32-bits of MSR behind %ecx into %eax. The higher
    # 32-bits are read into %edx.
    rdmsr
    or   $\bits, %eax
    wrmsr
.endm


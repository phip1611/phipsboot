# Calculates the real load address from the link address of a known symbol and
# stores it in the provided register. It relies on the relocation offset already
# was already set in %edx.
#
# Input Register:
#  - %edx - must contain the load offset
# Parameters:
#  - link_addr: Link address by its symbol name.
#  - reg_out: Register where the output is stored.
# Outputs:
#  - The register behind the parameter 'reg_out' will contain the result.
# Clobbers: reg_out
.macro M_RESOLVE_RUNTIME_ADDR link_addr reg_out
    mov     $\link_addr,    \reg_out
    add     %edx,           \reg_out
.endm

# Computes the page table offset for a virtual address with x86_64 4-level
# paging. The
# function returns the byte offset and not the number of the entry
# (nr of entry = offset / 4).
#
# For simplicity, this function only operates with 32-bit addresses. Hence,
# bits 63..32 are assumed to be all zero.
#
# Inputs:
#  - level    : 1, 2, 3, or 4
#  - vaddr_reg: Register that contains the virtual address to be mapped.
#  - reg_out  : Register where the output is stored.
# Outputs:
#   - The register behind the parameter 'reg_out' will contain the result.
# Clobbers: reg_out
.macro _M_CALC_PT_ENTRY_OFFSET level, reg_vaddr_lo, reg_out
    mov \reg_vaddr_lo,   \reg_out

    # Compile-time checks.
    .if \level < 1
        .error "Level must be 1, 2, 3, or 4."
    .endif
    .if \level > 4
        .error "Level must be 1, 2, 3, or 4."
    .endif

    # Select bits of current level.
    .if \level <= 2
        shr $(9 * (\level - 1) + 12), \reg_out
        and $0x1ff, \reg_out
    .endif
    .if \level == 3
        # Select the two highest bits.
        shr $30,  \reg_out
        and $0x3, \reg_out
    .endif
    .if \level == 4
        # As specified by macro description: bits 63..32 are all supposed to be zero.
        mov     $0, \reg_out
    .endif

    # Offset = index * size per entry
    imul    $8, \reg_out
.endm

/*
 * Calculates the offset into the page table for the given address.
 */
.macro M_CALC_PT_ENTRY_OFFSET level, vaddr, reg_out
    mov $\vaddr, \reg_out
    _M_CALC_PT_ENTRY_OFFSET  \level \reg_out \reg_out
.endm

/*
 * Calculates the offset into the page table for the given address but applies
 * the relocation offset to the address beforehand.
 */
.macro M_CALC_PT_ENTRY_OFFSET_RELOC level, vaddr, reg_out
    M_RESOLVE_RUNTIME_ADDR  \vaddr, \reg_out
    _M_CALC_PT_ENTRY_OFFSET  \level \reg_out \reg_out

.endm

.macro _M_CREATE_PT_ENTRY reg_entry_offset, curr_phys, dest_phys, entry_flags
    # Address of current page tabel entry = base addr + offset
    M_RESOLVE_RUNTIME_ADDR  \curr_phys, %ebx
    add %eax, %ebx # apply offset

    # Page table entry = target phys address + flags
    M_RESOLVE_RUNTIME_ADDR  \dest_phys, %ecx
    or $(\entry_flags & 0xfff), %ecx # add flags to entry

    # Write lower 32-bit of 64-bit page table entry
    mov %ecx, (%ebx)

    # Write upper half stays zero (just to be sure).
    movl $0, +4(%ebx)
.endm

/*
 * Create a page-table entry for the given address.
 */
.macro M_CREATE_PT_ENTRY vaddr, level, curr_phys, dest_phys, entry_flags
    M_CALC_PT_ENTRY_OFFSET  \level \vaddr, %eax
    _M_CREATE_PT_ENTRY  %eax, \curr_phys, \dest_phys, \entry_flags
.endm

/*
 * Create a page-table entry for the given address but applies the relocation
 * offset to the address beforehand.
 */
.macro M_CREATE_PT_ENTRY_RELOC vaddr, level, curr_phys, dest_phys, entry_flags
    M_CALC_PT_ENTRY_OFFSET_RELOC  \level \vaddr, %eax
    _M_CREATE_PT_ENTRY  %eax, \curr_phys, \dest_phys, \entry_flags
.endm


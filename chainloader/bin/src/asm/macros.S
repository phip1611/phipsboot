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
# Clobbers: rflags, reg_out
.macro  M_RESOLVE_RUNTIME_ADDR  link_addr reg_out
    mov  $\link_addr,  \reg_out
    add  %ebp,         \reg_out
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
 */
.macro  M_CALC_PT_ENTRY_OFFSET  level, vaddr
    mov  $\vaddr, %eax
    _M_CALC_PT_ENTRY_OFFSET  \level
.endm


/*
 * Calculates the offset into the page table for the given address but applies
 * the relocation offset to the address beforehand.
 */
.macro  M_CALC_PT_ENTRY_OFFSET_RELOC  level, vaddr
    M_RESOLVE_RUNTIME_ADDR   \vaddr, %eax
    _M_CALC_PT_ENTRY_OFFSET  \level
.endm


.macro  _M_CREATE_PT_ENTRY  curr_phys, dest_phys, entry_flags
    # Address of current page tabel entry = base addr + offset
    M_RESOLVE_RUNTIME_ADDR  \curr_phys, %ecx
    add  %eax, %ecx # add entry offset

    # Page table entry = target phys address + flags
    M_RESOLVE_RUNTIME_ADDR  \dest_phys, %eax
    or  $(\entry_flags & 0xfff), %eax

    # Write lower 32-bit of 64-bit page table entry
    mov  %eax, (%ecx)

    # Write upper half: stays zero (just to be sure).
    movl  $0, +4(%ecx)
.endm


/*
 * Create a page-table entry for the given address.
 */
.macro  M_CREATE_PT_ENTRY  vaddr, level, curr_phys, dest_phys, entry_flags
    M_CALC_PT_ENTRY_OFFSET  \level \vaddr
    _M_CREATE_PT_ENTRY  \curr_phys, \dest_phys, \entry_flags
.endm


/*
 * Create a page-table entry for the given address but applies the relocation
 * offset to the address beforehand.
 *
 */
.macro  M_CREATE_PT_ENTRY_RELOC  vaddr, level, curr_phys, dest_phys, entry_flags
    M_CALC_PT_ENTRY_OFFSET_RELOC  \level \vaddr
    _M_CREATE_PT_ENTRY  \curr_phys, \dest_phys, \entry_flags
.endm


/*
 * Adds the given bits to the given control register.
 */
.macro  M_SET_CR_BITS  reg_cr, bits
    mov  \reg_cr, %eax
    or   $\bits,  %eax
    mov  %eax,    \reg_cr
.endm


/*
 * Adds the given bits to the given MSR. Only uses the lower 32-bits.
 *
 * Clobbers: %ecx, %edx, %eax
 */
.macro  M_SET_MSR_BITS  msr, bits
    mov  $\msr, %ecx
    # reads the lower 32-bits of MSR behind %ecx into %eax. The higher
    # 32-bits are read into %edx.
    rdmsr
    # Enable 64-bit long mode.
    or   $\bits, %eax
    wrmsr
.endm


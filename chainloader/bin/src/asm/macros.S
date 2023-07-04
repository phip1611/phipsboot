# Calculates the real load address from the link address of a known symbol and stores
# it in the provided register. It relies on that the relocation offset was already set
# in %edx.
#
# Input Register:
#  - %edx - must contain the load offset
# Parameters:
#  - link_addr: Link address by its symbol name.
#  - out_reg: Register where the output is stored.
# Outputs:
#  - The register behind the parameter 'out_reg' will contain the result.
# Clobbers: out_reg
.macro M_RESOLVE_RUNTIME_ADDR link_addr out_reg
    mov     $\link_addr,    \out_reg
    add     %edx,           \out_reg
.endm

# Computes the offset into a page table of the given level for a virtual
# address with 32-bit PAE paging. The function returns the byte offset and not
# the number of the entry (nr of entry = offset / 4).
#
# Inputs:
#  - level    : 1, 2, or 3 (specifies which part of the virtual address is used).
#  - vaddr_reg: Register that contains the virtual address used for this calculation.
#  - out_reg  : Register where the output is stored.
# Outputs:
#   - The register behind the parameter 'out_reg' will contain the result.
# Clobbers: out_reg
.macro M_CALC_PT_ENTRY_INDEX level, vaddr_reg

    # We bit-shift the virtual address to the right.
    # *  9 - as each level is indexed by 9 bits (with x86 PAE paging)
    # + 12 - used as index into the page
    shr     $((\level - 1) * 9 + 12),   \vaddr_reg


    # Select bits of current level.
    .if \level <= 2
        and     $0x1ff,         \vaddr_reg
    .endif
    .if \level == 3
        and     $0x3,         \vaddr_reg
    .endif

    # Offset = index * size per entry
    imul    $8,             \vaddr_reg

.endm

.macro _M_MAP_PT_ENTRY vaddr_req, level, curr_phys, dest_phys, entry_flags
    # page table entry offset in %eax
    mov     \vaddr_req,           %eax
    M_CALC_PT_ENTRY_INDEX \level, %eax

    # Address of current page tabel entry = base addr + offset
    M_RESOLVE_RUNTIME_ADDR \curr_phys, %ebx
    add %eax, %ebx

    # Page table entry = target phys address + flags
    M_RESOLVE_RUNTIME_ADDR \dest_phys, %ecx
    or $(\entry_flags & 0xfff), %ecx # add flags to entry

    # Write lower 32-bit of 64-bit page table entry
    mov %ecx, (%ebx)
    # Upper half stays zero.
    movl $0, +4(%ebx)
.endm

.macro M_MAP_PT_ENTRY vaddr, level, curr_phys, dest_phys, entry_flags
    # page table entry offset in %eax
    mov     $\vaddr,    %eax
    _M_MAP_PT_ENTRY %eax, \level, \curr_phys, \dest_phys, \entry_flags
.endm

.macro M_MAP_PT_ENTRY_RELOC vaddr, level, curr_phys, dest_phys, entry_flags
    # page table entry offset in %eax
    M_RESOLVE_RUNTIME_ADDR \vaddr %eax
    _M_MAP_PT_ENTRY %eax, \level, \curr_phys, \dest_phys, \entry_flags
.endm

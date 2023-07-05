# Variables for the assembler file.
# Unfortunately, they end up as symbols in the final ELF's symbol table.
# This has no real downside, however.

# The register address of the MSR_IA32_EFER register.
.set MSR_IA32_EFER_REG, (0xc0000080)
# Bit that turns on 64-bit (IA32e - extended) mode.
.set MSR_IA32_EFER_LME, (1 << 8)


.set CR0_PE, (1 << 0)  # Protection Enable
.set CR0_WP, (1 << 16) # Write Protect
.set CR0_PG, (1 << 31) # Paging
.set CR0_BITS, (CR0_PE | CR0_WP | CR0_PG)


.set CR4_PAE, (1 << 5) # Physical Address Extension

# op-code of the "ljmp" instruction.
.set X86_LJMP, 0xea

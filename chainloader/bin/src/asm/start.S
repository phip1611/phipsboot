# The assembly file uses GNU Assembly (GAS) language with AT&T syntax.

# Entry of the bootloader. Look at "start" symbol for more information.

.extern rust_entry STACK_TOP

.code32
.section .boot.text, "ax", @progbits

# Entry into the loader. This "boot code" is fully position-independent. This
# way, the loader can cope with physical relocations when it is loaded by a
# Multiboot2 loader (such as GRUB) even though the ELF file is a static
# executable without any relocation information. Where necessary, instructions
# are patched during runtime to cope with relocations.
#
# Register Usage:
# - %edi holds the Multiboot2 magic
# - %esi holds the MBI address.
# - %ebp will contain the load address offset during runtime
# - %esp will be set to 0 prevent accidental stack usage in boot code
#   (but will be initialized before jumping into Rust code)
# - All other 32-bit registers can be used as general purpose registers
.global start
start:
    # #############################################################
    # Clear interrupts. They should never be activated at this point anyway.
    cli

    # #############################################################
    # Assert Multiboot2 handoff
    cmp  $MULTIBOOT2_MAGIC,  %eax
    je   0f # jump on success
    mov  $0xbadb001, %eax
    ud2  # crash system
    0:
    # flow continues..

    # #############################################################
    # Save Multiboot2 handoff parameters
    # (%eax and %ebx are working registers in the following)
    mov  %eax, %edi
    mov  %ebx, %esi

    # #############################################################
    # Find relocation offset and store it in %ebp

    mov   (%ebx), %eax   # Save first 4 bytes of MBI.
    mov   %ebx,   %esp   # Use MBI (= valid RAM) as temporary stack.
    call  1f             # call-pop trick -> get instruction pointer
    1:
    pop  %ebp
    sub  $1b,     %ebp   # %ebp now contains load offset (i32, i.e., pos or neg)
                         # that can be added to known (link = load address)-
                         # addresses to get the actual address.
    mov  %eax,    (%ebx) # Restore first 4 bytes of MBI.

    # #############################################################
    # Prepare page tables
    #
    # Flags:
    # -  0x3: present + writeable
    # - 0x83: present + writeable, huge page
    #
    # Structure:
    # L4 Table --> L3 Table ---> L2 Table --> 2 MiB identity mapping

    # Step 1/2: Identity mapping of actual load address.

    M_CREATE_PT_ENTRY_RELOC  0x200000, 4, boot_mem_page_table_l4, boot_mem_page_table_l3, 0x3
    M_CREATE_PT_ENTRY_RELOC  0x200000, 3, boot_mem_page_table_l3, boot_mem_page_table_l2_lo, 0x3
    M_CREATE_PT_ENTRY_RELOC  0x200000, 2, boot_mem_page_table_l2_lo, 0x200000, 0x83

    # Step 2/2: High-level mapping of Rust code

    # Map virtual address LINK_ADDR_LOADER to runtime address of
    # this binary as one single 2 MiB huge page mapping.

    # The symbol LINK_ADDR_LOADER is not available here for non-lazy
    # evaluation of the virtual address inside the macro.
    M_CREATE_PT_ENTRY  0x100000000, 4, boot_mem_page_table_l4, boot_mem_page_table_l3, 0x3
    M_CREATE_PT_ENTRY  0x100000000, 3, boot_mem_page_table_l3, boot_mem_page_table_l2_hi, 0x3
    M_CREATE_PT_ENTRY  0x100000000, 2, boot_mem_page_table_l2_hi, 0x200000, 0x83

    # #############################################################
    # Enable paging
    #
    # We bring the BSP from 32-bit protected mode without paging into IA32e
    # compatibility mode (64-bit mode with 32-bit opcodes) with 4-level paging.
    # The page table structure must be prepared accordingly beforehand. If no
    # identity mapping of this code at runtime exists, the BSP will die with a
    # page fault after the update of %cr0.

    M_RESOLVE_RUNTIME_ADDR  boot_mem_page_table_l4, %eax
    mov  %eax, %cr3 # set root page table

    M_SET_MSR_BITS  MSR_IA32_EFER_REG MSR_IA32_EFER_LME

    # Set paging properties.
    # Unlike for 32-bit paging, the CR4.PSE bit is not required for huge pages.
    M_SET_CR_BITS %cr4, CR4_PAE

    # Finally activate paging.
    M_SET_CR_BITS %cr0, CR0_BITS

    # Now, we are in 64-bit compatibility mode with 4-level paging. The code
    # running right now is identity mapped.

    # #############################################################
    # Far jump (ljmp) into 64-bit mode.

    # Update ldgdt input data with relocation offset and load GDT.
    M_RESOLVE_RUNTIME_ADDR  gdt,             %eax
    M_RESOLVE_RUNTIME_ADDR  gdt_ptr,         %ebx
    mov                     %eax,            (%ebx)
    M_RESOLVE_RUNTIME_ADDR ldt_gdt_data,     %eax
    lgdt                                     (%eax)

    M_RESOLVE_RUNTIME_ADDR bit64_trampoline, %eax
    M_RESOLVE_RUNTIME_ADDR ljmp_dest,        %ebx
    mov                     %eax,            (%ebx)

    # Equal to something like "ljmp $8:label" but constructed at runtime.
    # Required, as ljmp needs an absolute address.
    .byte X86_LJMP
    ljmp_dest: .long 0
    .word 8 # Use first selector of GDT (not the null selector)

    # #############################################################
    # 64-bit trampoline
    #
    # Prepare entry into Rust code: prepare function arguments
.code64
bit64_trampoline:
    # #############################################################
    # SystemV ABI x86_64 calling convention
    # 1st in %rdi, 2nd in %rsi, 3nd in %rdx

    # Boot Magic and Boot Info Ptr are already in %rdi and %rsi
    movsx  %ebp, %rdx  # 3rd param: Load offset: sign extend if negative

    # Setup stack + write stack canary at end of stack
    movabs  $STACK_TOP, %rsp
    sub  $16,  %rsp  # stack top is exclusive; prepare 16-byte stack alignment
    add  $8,   %rsp  # 16-byte stack alignment on x86 of first parameter

    movabs  $rust_entry, %rax
    jmp  *%rax # Absolute jump to link address of high-level code.

    ud2


.section .boot.data, "a", @progbits

.align 4
# Input data for the ldgdt instruction which takes a 6-byte sequence:
# the size (2 byte) and the location (4 byte) of the GDT.
ldt_gdt_data:
            .word gdt_end - gdt - 1
gdt_ptr:    .quad 0     # this location will be updated during runtime to reflect relocations


# Global Descriptor Table (GDT)
.align 8
gdt:
    # null selector
    .long 0
    .long 0

    # 64-bit code segment ring 0 selector
    .long 0
    .long 0xa09b00 # 64bit CS CPL0
gdt_end:

# 4K 8-byte aligned stack for the whole loader.
.align 16
# TODO MOVE STACK TO Rust and high link address
stack_begin:
    .zero 8192
stack_end:

# Page tables backing memory

.align 4096
.global boot_mem_page_table_l4
boot_mem_page_table_l4:
    .zero 4096

.align 4096
boot_mem_page_table_l3:
    .zero 4096

.align 4096
boot_mem_page_table_l2_hi:
    .zero 4096

.align 4096
boot_mem_page_table_l2_lo:
    .zero 4096

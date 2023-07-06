# The assembly file uses GNU Assembly (GAS) language with AT&T syntax.

# Entry code of the ELF binary. Look at "start" symbol for more information.

# Symbol from main.rs
.extern rust_entry

.code32
.align 8
.section .boot.text, "ax", @progbits

# Entry into the loader. This "boot code" is fully position-indepentent. This
# way, the loader can cope with relocations when it is loaded by a Multiboot2
# loader (such as GRUB).
# Where necessary, instructions are patched to cope with relocations.
#
# Register Usage:
# - %ebx stays intact to keep the MBI address.
# - %ebp will contain the load address offset during runtime
# - %esp will stay unused to prevent accidental stack usage
# - all other 32-bit registers can be used as general purpose registers
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

    mov  $0,      %esp   # Prevent usage of stack by accident.

    # #############################################################
    # Prepare page tables
    #
    # Flags:
    # -  0x3: present + writeable
    # - 0x83: present + writeable, huge page
    #
    # Structure:
    # L4 Table ---> L3 Table ---> L2 Table ("Lo") --> 2 MiB identity mapping
    #                        \--> L2 Table ("Hi") --> 2 MiB high-level code mapping
    #
    # Note: This mapping highly depends on LINK_ADDR_LOADER.

    # Step 1/2: Identity mapping of boot code

    # Map runtime address of LOAD_ADDR_BOOT to runtime address of
    # this binary as one single 2 MiB huge page mapping.
    M_CREATE_PT_ENTRY_RELOC  LOAD_ADDR_BOOT, 4, boot_mem_page_table_l4, boot_mem_page_table_l3, 0x3
    M_CREATE_PT_ENTRY_RELOC  LOAD_ADDR_BOOT, 3, boot_mem_page_table_l3, boot_mem_page_table_l2_lo, 0x3
    M_CREATE_PT_ENTRY_RELOC  LOAD_ADDR_BOOT, 2, boot_mem_page_table_l2_lo, LOAD_ADDR_BOOT, 0x83

    # Step 2/2: High-level mapping of Rust code

    # Map virtual address LINK_ADDR_LOADER to runtime address of
    # this binary as one single 2 MiB huge page mapping.

    M_CREATE_PT_ENTRY  LINK_ADDR_LOADER, 4, boot_mem_page_table_l4, boot_mem_page_table_l3, 0x3
    M_CREATE_PT_ENTRY  LINK_ADDR_LOADER, 3, boot_mem_page_table_l3, boot_mem_page_table_l2_hi, 0x3
    M_CREATE_PT_ENTRY  LINK_ADDR_LOADER, 2, boot_mem_page_table_l2_hi, LOAD_ADDR_BOOT, 0x83

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
    # Prepare entry into Rust code:
    # - setup stack
    # - prepare function arguments
.code64
bit64_trampoline:
    # Setup stack + write stack canary
    xor  %rsp, %rsp
    M_RESOLVE_RUNTIME_ADDR  stack_end, %esp
    movl  $0x13371337, -(stack_end - stack_begin - 4)(%rsp) # write canary 1/2
    movl  $0xdeadbeef, -(stack_end - stack_begin)(%rsp)      # write canary 2/2
    sub   $16,  %rsp # stack top is exclusive; prepare 16-byte stack alignment
    add   $8,   %rsp # 16-byte stack alignment on x86 of first parameter
    mov   %rsp, %rbp # set stack frame (probably useless on x86_64)

    # SystemV ABI / x86_64 calling convention:
    xor   %rdi, %rdi
    xor   %rsi, %rsi
    mov   %rbx, %rsi # 1st: MBI pointer
    movsx %ebp, %rdx # 2nd: Load offset: sign extend if negative

    mov  $rust_entry, %rax
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
stack_begin:
    .zero 8192
stack_end:

# Page tables backing memory

.align 4096
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

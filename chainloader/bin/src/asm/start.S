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
.global start
start:
    # Find relocation offset
    #
    # We use the MBI memory as temporary stack for call-pop trick to get IP.
    # We know that the relocation offset is a multiple of 2 MiB due our
    # alignment requirement specified in the MB2 header.

    mov     (%ebx), %ecx    # save first 4 MBI bytes
    mov     %ebx,   %esp
    call    1f
    1: pop     %edx
    sub     $1b,    %edx    # %edx now contains load offset that can be added to
                            # link addresses to get the actual address
    mov     %ecx,   (%ebx)  # restore first 4 MBI bytes

    # #############################################################
    # Prepare stack

    mov     $stack_end,    %esp    # set stack end (stack grows downwards)
    sub     $16,           %esp    # 16-byte stack alignment on x86
    add     $8,            %esp    # the first parameter that is passed via the
                                   # stack must be 16-byte aligned
    add     %edx,          %esp    # apply relocation offset to get actual stack
    mov     %esp,          %ebp    # set stack frame (important on i386)

    # #############################################################
    # Save %eax (MB2 Magic) and %ebx (MBI) on stack to free registers.
    # Now, all 32-bit registers are usable except for %edx, which holds the
    # load offset.
    push    %eax
    push    %ebx

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
    mov     %eax,   %cr3 # load root page table

    # Save relocation offset. This register is clobbered by rdmsr.
    push %edx

    mov     $MSR_IA32_EFER_REG,    %ecx
    # reads the lower 32-bits of MSR behind %ecx into %eax. The higher
    # 32-bits are read into %edx.
    rdmsr

    # Enable 64-bit long mode.
    or      $MSR_IA32_EFER_LME,    %eax
    wrmsr

    # Restore relocation offset.
    pop %edx

    # Set paging properties.
    # Unlike for 32-bit paging, the CR4.PSE bit is not required for huge pages.
    mov     %cr4,      %eax
    or      $CR4_PAE,  %eax
    mov     %eax,      %cr4

    # Activate paging.
    mov     %cr0,      %eax
    or      $CR0_BITS, %eax
    mov     %eax,      %cr0

    # Now, we are in 64-bit compatibility mode with 4-level paging.
    # Next, we need a far jump/long jump into 64-bit mode.

    # #############################################################
    # Far jump (ljmp) into 64-bit mode.

    # Restore previously saved values. Don't clobber these registers now!
    pop %ebx
    pop %eax

    # Update ldgdt input data with relocation offset and load GDT.
    M_RESOLVE_RUNTIME_ADDR gdt,     %edi
    M_RESOLVE_RUNTIME_ADDR gdt_ptr, %esi
    mov %edi, (%esi)
    M_RESOLVE_RUNTIME_ADDR ldt_gdt_data, %edi
    lgdt    (%edi)

    M_RESOLVE_RUNTIME_ADDR bit64_trampoline, %edi
    M_RESOLVE_RUNTIME_ADDR ljmp_dest,        %esi
    mov %edi, (%esi)

    # Equal to something like "ljmp $8:label" but constructed at runtime.
    # Required, as ljmp needs an absolute address.
    .byte X86_LJMP
    ljmp_dest: .long 0
    .word 8 # Use first selector of GDT (not the null selector)

    # #############################################################
    # 64-bit trampoline
    #
    # Prepare function arguments for rust entry
.code64
bit64_trampoline:

    # SystemV ABI / x86_64 calling convention:
    xor     %edi, %edi
    xor     %esi, %esi
    mov     %rax, %rdi # MB2 magic
    mov     %rbx, %rsi # MBI pointer

    mov     $rust_entry, %rax
    # Absolute jump to link address of high-level code.
    call    *%rax
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

# The assembly file uses GNU Assembly (GAS) language with AT&T syntax.

# Entry into the binary. Prepares jumping into Rust code.

# Symbol from main.rs
.extern rust_entry

.code32
.align 8
.section .boot.text, "ax", @progbits

.global start
start:
    # Find relocation offset
    # We use the MBI memory as temporary stack for call-pop trick to get IP.
    # We know that the relocation offset is a multiple of 2 MiB due our
    # alignment requirement specified in the MB2 header.

    mov     (%ebx), %ecx    # save first 4 MBI bytes
    mov     %ebx,   %esp
    call    1f
    1:
    pop     %edx
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

    # Step 1/2: Identity mapping of boot code

    M_MAP_PT_ENTRY_RELOC LOAD_ADDR_BOOT, 3, boot_mem_page_table_l3, boot_mem_page_table_l2_lo, 0x3
    M_MAP_PT_ENTRY_RELOC LOAD_ADDR_BOOT, 2, boot_mem_page_table_l2_lo, LOAD_ADDR_BOOT, 0x83

    # Step 2/2: High-level mapping of Rust code
    M_MAP_PT_ENTRY LINK_ADDR_LOADER, 3, boot_mem_page_table_l3, boot_mem_page_table_l2_hi, 0x3
    M_MAP_PT_ENTRY LINK_ADDR_LOADER, 2, boot_mem_page_table_l2_hi, LOAD_ADDR_BOOT, 0x83

    # #############################################################
    # Enable paging

    mov     $boot_mem_page_table_l3, %eax
    add     %edx,   %eax # %eax contains the runtime address of the root page table
    mov     %eax,   %cr3 # load root page table


.set CR0_PE, (1 << 0)  # Protection Enable
.set CR0_WP, (1 << 16) # Write Protect
.set CR0_PG, (1 << 31) # Paging
.set CR0_BITS, (CR0_PE | CR0_WP | CR0_PG)

.set CR4_PSE, (1 << 4) # Page Size Extension
.set CR4_PAE, (1 << 5) # Physical Address Extension
.set CR4_BITS, (CR4_PSE | CR4_PAE)

    mov     %cr4,      %eax
    or      $CR4_BITS, %eax
    mov     %eax,      %cr4

    mov     %cr0,      %eax
    or      $CR0_BITS, %eax
    mov     %eax,      %cr0 # now, paging is active

    # #############################################################
    # Prepare function arguments for rust entry

    # Restore previously saved values.
    pop %ebx
    pop %eax

    # x86 (i386) calling convention:
    # 1. prepare stackframe pointer
    # 2. push arguments on stack in reverse order
    push    $0x0
    push    %ebp
    mov     %esp,   %ebp
    push    %edx    # Relocation offset
    push    %ebx    # MBI pointer
    push    %eax    # Multiboot2 magic

    mov     $rust_entry, %eax
    # Absolute jump to link address of high-level code.
    call    *%eax
    ud2


.section .boot.data, "a", @progbits

# 4K 8-byte aligned stack for the whole loader.
.align 16
stack_begin:
    .zero 8192
stack_end:

# Page tables backing memory required for 32-bit PAE paging with
# one single 2 Mib huge mapping.

.align 4096
boot_mem_page_table_l3:
    .zero 4096

.align 4096
boot_mem_page_table_l2_lo:
    .zero 4096

.align 4096
boot_mem_page_table_l2_hi:
    .zero 4096

# The assembly file uses GNU Assembly (GAS) language with AT&T syntax.

# Entry into the binary. Prepares jumping into Rust code.

# Symbol from main.rs
.extern rust_entry

.code32
.align 8
.section .text

.global start
start:
    cli

    # Find relocation offset
    # We use memory the MBI as temporary stack for call-pop trick to get IP.

    mov     %ebx,   %esp
    mov     (%ebx), %ecx    # save first 4 MBI bytes

    call    1f
    1:
    pop     %edx
    sub     $1b,    %edx    # edx now contains load offset that can be added
                            # link addresses to get the actual address
    mov     %ecx,   (%ebx)  # restore first 4 MBI bytes

    ##############################################################
    # Prepare stack

    mov     $stack_end, %esp # set stack end (stack grows downwards)
    sub     $4,         %esp # stack end is exclusive / invalid memory
    add     %edx,       %esp # apply relocation offset
    mov     %esp,       %ebp

    # x86 (i386) calling convention:
    # 1. prepare stackframe pointer
    # 2. push arguments on stack in reverse order
    push    %ebp
    mov     %esp,   %ebp
    push    %eax    # Multiboot2 magic
    push    %ebx    # MBI pointer
    push    %edx    # Load offset

    #mov     $rust_entry,    %ecx
    #add     %edx,    %ecx
    #jmp    *%ecx

    cli
    hlt

    call rust_entry

    ud2

.section .data

# 4K 8-byte aligned stack.
.align 8
stack_begin:
    .zero 4096
stack_end:

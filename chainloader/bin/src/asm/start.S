# The assembly file uses GNU Assembly (GAS) language with AT&T syntax.

# Entry into the binary. Prepares jumping into Rust code.

# Symbol from main.rs
.extern rust_entry

.code32
.align 8
.section .text

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

    ##############################################################
    # Prepare stack

    mov     $stack_end, %esp    # set stack end (stack grows downwards)
    sub     $16,        %esp    # 16-byte stack alignment on x86
    add     $8,         %esp    # the first parameter that is passed via the
                                # stack must be 16-byte aligned
    add     %edx,       %esp    # apply relocation offset to get actual stack
    mov     %esp,       %ebp    # set stack frame (important on i386)

    ##############################################################
    # Prepare function arguments for rust entry

    # x86 (i386) calling convention:
    # 1. prepare stackframe pointer
    # 2. push arguments on stack in reverse order
    push    $0x0
    push    %ebp
    mov     %esp,   %ebp
    push    %edx    # Relocation offset
    push    %ebx    # MBI pointer
    push    %eax    # Multiboot2 magic

    call rust_entry     # relative instruction
    ud2


.section .data

# 4K 8-byte aligned stack.
.align 16
stack_begin:
    .zero 4096
stack_end:

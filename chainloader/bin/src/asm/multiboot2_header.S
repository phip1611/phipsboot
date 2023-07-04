# The assembly file uses GNU Assembly (GAS) language with AT&T syntax.

# Multiboot2 Header definition.

.align 8
.section .mb2_hdr, "a", @progbits

                        .align 8 /* Header must be 8-byte aligned. */
multiboot2_hdr_start:   .long   0xe85250d6
                        .long   0x0 /* arch: i386 */
                        .long   (multiboot2_hdr_end - multiboot2_hdr_start)
                        .long  -(0xe85250d6 + (multiboot2_hdr_end - multiboot2_hdr_start))

                        /* module alignment tag */
                        .align 8
                        .word   0x6
                        .word   0x0 /* flags */
                        .long   0x8 /* size */

                        /* information request tag */
                        .align 8
                        .Linformation_request_begin:
                        .word   0x1
                        .word   0x1 /* flags: optional */
                        .long   (.Linformation_request_end - .Linformation_request_begin) /* size */
                        .long  1
                        .long  2
                        .long  3
                        .long  4
                        .long  5
                        .long  6
                        .long  7
                        .long  8
                        .long  9
                        .long  10
                        .long  11
                        .long  12
                        .long  13
                        .long  14
                        .long  15
                        .long  16
                        .long  17
                        .long  18
                        .long  19
                        .long  20
                        .long  21
                        .Linformation_request_end:

                        /* end tag */
                        .align 8
                        .word   0x0 /* type */
                        .word   0x0 /* flags */
                        .long   0x8 /* size */
multiboot2_hdr_end:

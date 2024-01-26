# Architecture

This document describes the architecture of the binary and the Cargo project.

## Relocatable

The whole architecture is completely influenced by the physically
relocatable property. Being relocatable at boot is very important to prevent
clashing with MMIO regions or already occupied RAM. There is not a single
physical address range being valid RAM that is guaranteed to be available on all
x86 platforms worldwide. Thus, being relocatable is key for a bootloader.
Building a kernel (or bootloader) binary with fully position-independent code
using a high-level language is not trivial, unfortunately. A truly
position-independent ELF binary needs relocation information embedded into the
ELF. GRUB is not able to use these relocations. Producing a static ELF with
position independent code might work for a few corner cases but the code can
produce undefined behaviour in unpredictable situations.

A solution is what this chainloader implements: The binary itself consists of
assembly code ("boot code") and actual high-level code ("loader code"). As I
write the assembly code myself, I have full control to only use
position-independent code. Code needing absolute addressing can be patched at
runtime, as done for a few places inside the assembly files.

The actual loader code is linked at a certain virtual address. The boot code
prepares paging and eventually jumps into the loader code. Due to paging, the
high-level loader code does not mind where it is in physical memory.

Finally, the loader code can load a further ELF at any wanted address but with a
handoff in 64-bit long mode.

Again, on UEFI platforms, you should just use a custom EFI loader. EFI
applications are designed to be relocatable by design. The EFI loader inside the
firmware can cope with that. The complicated approach chosen by me is just
necessary as GRUB is not capable of applying relocations.

More Info:
- <https://github.com/rust-lang/rust/issues/113207>
- <https://twitter.com/phip1611/status/1675541476913152001>


## Binary

The loader produces a static binary in x86_64 ELF format with four LOAD segments
and no relocation information. They are all continuous in physical memory and
all together less than two MiB in size. They must be loaded at a two MiB aligned
physical (and virtual) address. The loader is relocatable in physical address
space but not in virtual address space. All symbols in the loader have the same
offset regarding the previous 2 MiB boundary in physical and their corresponding
virtual address space.

### Load Segment Overview

- `boot`: Consists of code that was written in assembly. It holds the entry
  point into the binary. It is position-independent and patches its code live
  during runtime to cope with a relocation, if necessary.
- `rx`: Contains the compiled instruction stream from the compiled Rust code.
- `ro`: Contains read-only symbols from the compiled Rust code.
- `rw`: Contains writeable data symbols from the compiled Rust code.

### Two MiB Restriction

Having a two MiB alignment requirement allows to use two MiB huge-mappings for
the LOAD segments. Having an overall size-restriction of two MiB allows to map
the whole loader using one single L1 page table that covers the whole range.

Furthermore, the less virtual memory space the binary allocates, the more
freedom do kernel payloads of the loader have.

### Page Table Structure

Two facilitate the transition to 64-bit mode, we need an identity mapping of the
boot code and a regular mapping of the actual code. The binary is structured
in a way to ease the following mapping structure.

For reference: L4, L3, L2, and L1 correspond to the page-table levels of
64-bit 4-level paging.

```
L4 -> L3 (lo) -> L2 (lo) -> 2 MiB r+x idendity mapping of boot code
  \-> L3 (hi) -> L2 (hi) -> L1 (hi) -> x rx mappings of RX segment
                                   \-> x ro mappings of RO segment
                                   \-> x rw mappings of RW segment
```

## Machine State Transitions

- Phase 1/3: "boot code" (written in assembly)
    - _I386 machine state_ (see Multiboot2 spec)
    - Entry into the relocatable portion of the chainloader ("boot code").
    - Sets up 64-bit 4-level paging.
        - Identity mapping for boot code (2 MiB huge page).
        - Link-address mapping for high-level loader code (2 MiB huge page).
    - Jump into 64-bit long mode.
    - Jump into Rust code (phase 2)
- Phase 2/3: "high-level loader code"
    - Find binary to load in the Multiboot2 boot information.
    - Map kernel using 2 MiB huge mappings.
    - Prepare handoff information.
    - Jump to kernel (pase 3)
- Phase 3/3: "kernel code"
    - PhipsBoot is done
    - Your kernel takes over control.


## Cargo Project

The project tries to facilitate as much as possible of the Cargo-native
toolchain options to build this binary. However, for a convenient testing and
build environment - `no_std` binaries + Cargo come in fact with some caveats -
a small Makefile-based wrapper is used.

### Why a Custom Rustc Target?

TODO

## High-level Rust Code

The high-level Rust code is compiled to 64-bit code and expects a bootstrapped
CPU. This means activated paging and all required CPU features that are
used during compilation, for example `SSE`. The boot code is responsible for
preparing this environment.

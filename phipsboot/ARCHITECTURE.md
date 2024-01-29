# Architecture

This document describes the architecture of the binary and the Cargo project.

## General Properties

The whole architecture is the project, the toolchain, and the code is focused
on creating a static ELF executable that is **relocatable** in physical memory,
when it is booted in 32-bit protected mode without paging. This is also the boot
process chosen by projects such as
[Hedron](https://github.com/cyberus-technology/hedron) or
[NOVA](https://hypervisor.org/). A comprehensive writeup about the background
properties and more background info can be found here:
- <https://phip1611.de/blog/x86-kernel-development-relocatable-binaries>
- <https://github.com/rust-lang/rust/issues/113207>
- <https://twitter.com/phip1611/status/1675541476913152001>

## Binary

PhipsBoot is an x86_64 binary packaged as static executable ELF. It has four
LOAD segments and no relocation information. All LOAD segments are continuous in
physical memory and all together `<= 2 MiB` in size for easier page-table
management. They must be loaded at a two MiB aligned physical (and virtual)
address.

The binary consists of the "boot code" and the "loader code". The boot code
is written in assembly and position-independent. It does so by using only
relative addressing and live binary patching where absolute addresses are
required. It prepares the page-tables to jump into the loader code. The loader
code is written in Rust.

### Load Segment Overview

- `boot`: Consists of code that was written in assembly. It holds the entry
  point into the binary. It is position-independent and patches its code live
  during runtime to cope with a relocation, if necessary.
- `rx`: Contains the compiled instruction stream from the compiled Rust code.
- `ro`: Contains read-only symbols from the compiled Rust code.
- `rw`: Contains writeable data symbols from the compiled Rust code.

It would be possible to just use a `boot` segment and a `loader` segment with
all the permissions. However, for safety, security, and correctness, and also
to catch bugs, I refrained from that approach.

### Page Table Structure

The boot code sets up the complete following page-table structure, before it
activates paging. L4, L3, L2, and L1 correspond to the page-table levels of
64-bit 4-level paging:

```
L4 -> L3 (lo) -> L2 (lo) -> 2 MiB r+x identity mapping of boot code
  \-> L3 (hi) -> L2 (hi) -> L1 (hi) -> x rx mappings of RX segment
                                   \-> x ro mappings of RO segment
                                   \-> x rw mappings of RW segment
```

### Two MiB Restriction

Having a two MiB alignment requirement allows to one single L2 table to use on
single 2 MiB huge-page mapping (done for the identity mapping) and one single
L2 table to map all the other segments.

Furthermore, the less virtual memory space the binary allocates, the more
freedom do kernel payloads of the loader have. 2 MiB is fairly a lot for a
bootloader.

## Machine State Transitions

- Phase 1/3: "boot code" (written in assembly)
    - _I386 machine state_ (see Multiboot2 spec)
    - Entry into "boot code".
    - Sets up paging
    - Jump into 64-bit long mode.
    - Jump into Rust code (phase 2)
- Phase 2/3: "high-level loader code"
    - Find kernel to boot.
    - Load it into kernel.
    - Prepare handoff information.
    - Jump to kernel (phase 3)
- Phase 3/3: "kernel code"
    - PhipsBoot is done
    - Your kernel takes over control.

## Cargo Project

The project tries to facilitate as much as possible of the Cargo-native
toolchain options to build this binary. However, for a convenient testing and
build environment - `no_std` binaries + Cargo come in fact with some caveats -
a small Makefile-based wrapper is used.

### Why a custom rustc target?

[More information.](bin/x86_64-unknown-none-static.json.README.md).

## High-level Rust Code

The high-level Rust code is compiled to 64-bit code and expects a bootstrapped
CPU. This means activated paging and all required CPU features that are
used during compilation, for example `SSE`. The boot code is responsible for
preparing this environment.

# PhipsBoot

⚠️ Work in progress. This is more proof-of-concept than production-ready. ⚠️

Disclaimer: _This project combines a lot of toolchain and binary knowledge and
experience I collected and gained in recent years about legacy x86_64 boot. The
main contribution IMHO is how the binary is assembled and that the thing boots
with all the properties described below, but not the high-level functionality
itself._

_I am especially proud of the well-commented structure of the assembly files.
For example the whole page-table mappings are done IMHO very nicely even though
it is assembly language. Also, I think it turned out quite cool how I configured
the linker script. I hope this can be a learning resource for others!_

_Further: This project is a solution for a niche use-case, especially in 2024._

PhipsBoot is a relocatable x86_64 bootloader written in Rust and assembly that
loads a kernel into 64-bit mode. It abstracts a lot of boot-related x86_64
complexity away.

The main advantage of PhipsBoot is seen when loaded by GRUB via Multiboot2 in
legacy BIOS boot systems, where it can be relocated in physical memory even
though the kernel binary is a static ELF. However, PhipsBoot also supports
Multiboot1 and XEN PVH entries.

## About

### Supported Boot Environments

PhipsBoot expects an 32-bit protected mode without paging machine state at is
entry. This corresponds to the Multiboot2 i386 machine state definition.

| Firmware  | Hand-Off by   | Status | Comment                                 |
|-----------|---------------|--------|-----------------------------------------|
| BIOS/UEFI | Firmware      | ❌      | Use existing projects as chainloader.   |
| BIOS/UEFI | Multiboot 1/2 | ✅      | Recommended (BIOS + GRUB + Multiboot 2) |
| *         | Xen PVH       | ✅      |                                         |


### Why Relying On GRUB + Multiboot2?

Effectively, in the open-source world you are limited to GRUB when you want to
boot your custom kernel on legacy BIOS x86_64 systems, while keeping up a
certain level of user and developer experience. By reusing all the abstracted
away complexity that GRUB comes with, PhipsBoot can be much simpler and easier
to program, install, adopt, and use.

### Which problems does PhipsBoot solve?

It solves several problems every 64-bit kernel has to perform anyway, such as
performing the transition into 64-bit long mode while also being relocatable in
physical memory to never clash with firmware code, MMIO regions or otherwise
reserved addresses. Kernels using this bootloader can massively simplify their
build setup and boot code as much complexity is outsourced to PhipsBoot, such as
several CPU state transitions and intermediate identity page mappings.

By far the biggest contribution of PhipsBoot is that it is relocatable in
physical memory when it is loaded by GRUB. Here, you can read more of the
[overall challenges](https://phip1611.de/blog/x86-kernel-development-relocatable-binaries/).

### Related Projects & Practical Alternatives

First of all: If you are targeting UEFI and only UEFI, you need none of this!
The world is much simpler there. However, for legacy BIOS, the bootloader
developer and user experience is just difficult. To not be forced to use GRUB,
one can also write an
[entire bootloader supporting legacy BIOS boot in Rust](https://github.com/rust-osdev/bootloaderz).
That's awesome! However, installing legacy BIOS stage 1 bootloaders on
disk is much more complicated, as one has to patch the MBR instead of just
putting a file on disk. By using GRUB however, it is relatively easy to put
PhipsBoot or other Multiboot payloads on disk and reference them from the GRUB
config.

If you are targeting legacy x86 boot and have the freedom to choose and install
your own stage 1 bootloader, you can also have a look at
[Limine](https://limine-bootloader.org/), which is superior to GRUB and
production ready. It also brings your kernel into 64-bit mode right away, making
PhipsBoot obsolete.

### Trivia

I'm kind of proud of the way I set up page-tables from assembly using
descriptive GNU as macros! I didn't see anything like that so far. IMHO it is
a great approach as low-level programming and assembly doesn't necessarily has
to be ugly. 🤓

## Developer Guide

- General description about the architecture: [ARCHITECTURE.md](phipsboot/ARCHITECTURE.md)
- Build and run test: `$ make && make integration-test`

Artifacts are in `./build`.

## User Guide

### Boot PhipsBoot (for testing)

You have multiple options, for example:

- `$ cloud-hypervisor --debug-console file=log.txt --kernel ./build/phipsboot.elf64` (using Xen PVH)
- `$ qemu-system-x86_64 -kernel ./build/phipsboot.elf32 -debugcon stdio` (using Multiboot 1)

### Supported Kernel Payloads

Supported payloads that PhipsBoot can boot are ELF executables (static and dyn).
The hand-off to the kernel follows the PhipsBoot protocol.

### PhipsBoot protocol

This protocol describes the hardware state and the handover to your kernel when
it is booted by PhipsBoot.

#### Kernel Entry

The kernel entry is taken from the ELF entry. It is invoked using the SystemV
x86_64 calling convention. First argument passed to your kernel is a point to
the boot information.

#### Machine State after hand-off

- PhipsBoot is still mapped and occupies (at most) 2 MiB of virtual address
  space
- BSP in 64-bit long mode with 4-level paging
- APs are still asleep
- control registers
    - `%cr0`: PE (0), MP (1), WP (16), PG (31)
    - `%cr4`: PAE (5), OSFXSR (9), OSXMMEXCPT (10)
    - `%cr3`: holds the physical address of the root page table
- MSRs
    - `efer`: LME (8), NX (11)
- GDT, which is living in the physical address space of PhipsBoot, is set with
  two selectors:
    - null selector
    - (64-bit, code segment, ring 0)-selector
- `%rsp` is set to a valid 128 KiB stack
- `%rdi` has pointer to boot information
- All load segments of the kernel are loaded with their corresponding page-table
  rights. The NX bits are set for all non-executable LOAD segments.

#### Boot Information

TODO implement

### Booting Your Kernel with PhipsBoot

TODO implement

You can use the following GRUB configuration:

```
menuentry "Kernel" {
    multiboot2 /phipsboot
    module2 /your-kernel
    boot
}
```

#### Binary Formats of PhipsBoot

The build itself produces `phipsboot.elf32` and `phipsboot.elf64`. Both are
identical except for the ELF header. You usually always want to use the `.elf64`
version except for when booting it via Multiboot1, where compliant bootloaders
only accept 32-bit ELFs.

<!--
TODO
Furthermore, the build also produces a `.iso` variant that is bootable on
legacy BIOS systems. The `.iso` variant uses a GRUB standalone image that
chainloads PhipsBoot via Multiboot 2. GRUB2 will physically relocate PhipsBoot.
The `.iso` variant is used for testing and for you as inspiration for on how
you can package PhipsBoot along with your kernel.
-->

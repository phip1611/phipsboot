# PhipsBoot

PhipsBoot is a relocatable x86_64 Multiboot2 chainloader recommended for usage
in legacy boot environments. It is intended to be chainloaded by GRUB via
Multiboot2 and mostly written in idiomatic Rust.

## TL;DR: What does PhipsBoot do?

It boots your x86_64 kernel in ELF format at its desired virtual address and
performs the handoff in 64-bit long mode.

## Which problems does PhipsBoot solve?

It solves several problems every 64-bit kernel has to perform anyway, such as
performing the transition into 64-bit long mode while also being relocatable in
physical memory to never clash with firmware code, MMIO regions or otherwise
reserved addresses. Kernels using this bootloader can massively simplify their
build setup and boot code as much complexity is outsourced to PhipsBoot, such as
several CPU state transitions and intermediate identity page mappings.

By far the biggest contribution of PhipsBoot is that it is relocatable in
physical memory but jumps into code compiled from a high-level language as soon
as possible. For that, you need position-independent code at the beginning and
to a certain degree also live-patching during runtime so that all instructions
find the data they need at the right place.

To my knowledge, only [NOVA](https://hypervisor.org/) and [Hedron](https://github.com/cyberus-technology/hedron)
perform a similar complicated setup to get all the flexibility of being
physically relocatable.

While it is easy to create a suited bootloader as EFI app on an UEFI platform,
on a legacy system you are most likely limited to GRUB with a Multiboot2 handoff
for a convenient boot flow for your OS project. This is where PhipsBoot helps
with all the benefits described above.

## Machine State before and after PhipsBoot is done

PhipsBoot starts in _I386 machine state_ (see Multiboot2 spec) and loads a
provided ELF binary (an actual kernel) into memory at its desired link address.
The kernel payload sees a handoff similar to the `I386 machine state` state,
except that the Bootstrap Processor (BSP) is in 64-bit long mode. Hence, the
handoff to the kernel can happen at a high address such as `0xffffffff88000000`
and your kernel doesn't need to do that transition to 64-bit long mode
and loading itself where to the location it wants to be all by itself.

## Handoff to your kernel

Your loaded kernel receives a boot information structure passed at handoff. This is similar
to the Multiboot2 information that the PhipsBoot receives by GRUB but
enhanced with more info about the load environment.

## Why Relying On Multiboot2 / GRUB?

It is a chainloader rather than a "full" bootloader to benefit from all the
complexity GRUB already takes away from us. GRUB is the most popular
Multiboot2-compatible bootloader out there. With my chainloader, every OS
project that wants to target legacy systems can just use this chainloader and
also reuse GRUB.

### Final Machine State

- BSP in 64-bit long mode with 4-level paging
- All load segments of the kernel are loaded via 2 MiB huge page mapping with
  their corresponding page-table rights.
- APs are still asleep
- Register `%rdi` has pointer to PhipsBoot boot information
  - This includes the memory map
- `CR0` has the following bits set: PE (0), WP (1), PG (31)
- `CR3` holds the physical address of the root page table
- `CR4` has the following bits set: PAE (5)
- GDT living in the address space of the loader is set with two selectors:
  null selector and a (64-bit, code segment, ring 0)-selector

## Supported Kernel Payloads & How Does PhipsBoot Find Your Kernel

TODO

```
menuentry "Kernel" {
    multiboot2 /PhipsBoot load=foo-kernel
    # This adds "foo-kernel" to the cmdline of this boot module and the loader
    # knows which file to load.
    module2 /foo-kernel foo-kernel
    boot
}
```

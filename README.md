# %PROJECT_NAME%

%PROJECT_NAME% is a relocatable x86 Multiboot2 chainloader recommended
for usage in legacy boot environments. It is intended to be chainloaded by GRUB
via Multiboot2 and mostly written in idiomatic Rust.

It solves the problem to load a kernel into the negative (high half) of the
address space and execute it from there, which needs a lot of setup, such as
CPU state transitions and intermediate identity page mappings. While it is easy
to create a suited bootloader as EFI app on an UEFI platform, on a legacy system
you are most likely limited to GRUB with a Multiboot2 handoff for a convenient
boot flow for your OS project. This is where %PROJECT_NAME% helps, as it is
fully relocatable by a Multiboot2 loader, so it can set up the CPU in a reliable
way.

%PROJECT_NAME% starts in _I386 machine state_ (see Multiboot2 spec) and loads a
passed ELF binary (an actual kernel) into memory at its desired link address.
The kernel payload sees a handoff similar to the `I386 machine state` state,
except that the Bootstrap Processor (BSP) is in 64-bit long mode. Hence, the
handoff to the kernel can happen at a high address such as `0xffffffff88000000`
and your kernel doesn't need to do that transition itself.

The kernel gets a boot information structure passed at handoff. This is similar
to the Multiboot2 information that the %PROJECT_NAME% receives by GRUB but
enhanced with more info about the load environment.

## Architecture: Being Relocatable
Being relocatable at boot is very important to prevent clashing with MMIO
regions or already occupied RAM. There is not a single physical address range
being valid RAM that is guaranteed to be available on all x86 platforms
worldwide. Thus, being relocatable is key for a bootloader. Building a kernel
(or bootloader) binary with fully position-independent code using a high-level
language is not trivial, unfortunately. A truly position-independent ELF binary
needs relocation information embedded into the ELF. GRUB is not able to use
these relocations. Producing a static ELF with position independent code might
work for a few corner cases but the code can produce undefined behaviour in
unpredictable situations.

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

### Why Relying On GRUB?
GRUB is the most popular (Multiboot2-capable) bootloader out there. With my
chainloader, every OS project that wants to target legacy systems can just use
this chainloader and reuse GRUB. Writing stage 1 bootloaders for legacy systems
is not trivial.

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
  - %PROJECT_NAME% is done
  - Your kernel takes over control.

### Final Machine State
- BSP in 64-bit long mode with 4-level paging
- All load segments of the kernel are loaded via 2 MiB huge page mapping with
  their corresponding page-table rights.
- APs are still asleep
- Register `%rdi` has pointer to %PROJECT_NAME% boot information
  - This includes the memory map
- `CR0` has the following bits set: PE (0), WP (1), PG (31)
- `CR3` holds the physical address of the root page table
- `CR4` has the following bits set: PAE (5)
- GDT living in the address space of the loader is set with two selectors:
  null selector and a (64-bit, code segment, ring 0)-selector

## Supported Kernel Payloads
%PROJECT_NAME%  can load static ELF files. ELF files with relocations are in
the work. In any case: each load segment needs to be aligned to a 2 MiB boundary
for 2 MiB huge pages.

## How Does %PROJECT_NAME% Find Your Kernel
Pass the file as Multiboot2 boot module as shown in the following GRUB
configuration:

```
menuentry "Kernel" {
    multiboot2 /%PROJECT_NAME% load=foo-kernel
    # This adds "foo-kernel" to the cmdline of this boot module and the loader
    # knows which file to load.
    module2 /foo-kernel foo-kernel
    boot
}
```

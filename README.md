# %PROJECT_NAME%

%PROJECT_NAME% is a relocatable x86 Multiboot2 chainloader recommended
for usage in legacy boot environments. It is intended to be chainloaded by GRUB
via Multiboot2.

It solves the problem that you somehow want to load your kernel
into the negative (high half) of the address space and execute it from there,
which needs a lot of setup. While it is easy to create a suited bootloader as
EFI app on an UEFI platform, on a legacy system you are most likely limited to
GRUB with a Multiboot2 handoff. Furthermore, you want the component loaded by
GRUB to be relocatable. This is where  %PROJECT_NAME% helps you.

%PROJECT_NAME% starts in _I386 machine state_ (see Multiboot2 spec) and loads a
passed ELF binary (an actual kernel) into memory at its desired link address.

The kernel payload sees a handoff similar to the `I386 machine state` state,
except that the Bootstrap Processor (BSP) is in 64-bit long mode. Hence, the
handoff to the kernel can happen at a high address such as `0xffffffff88000000`.

The kernel gets a %PROJECT_NAME% boot information passed. This is similar to the
Multiboot2 information that the %PROJECT_NAME% receives by GRUB but enhanced
with more info.

## Machine State Transitions
- Phase 1/3: "boot code" (written in assembly)
  - _I386 machine state_ (see Multiboot2 spec)
  - Entry into the relocatable portion of the chainloader ("boot code").
  - Sets up paging with Physical Address Extension (PAE) (2 bit, 9 bit, 9 bit)
    and Page Size Extension (PSE) (2 MiB huge pages).
    - Identity mapping for boot code.
    - Link-address mapping for high-level loader code.
  - Jumps to phase 2
- Phase 2/3: "high-level loader code"
  - Find binary to load in the Multiboot2 boot information.
  - Load binary into memory.
  - Map kernel using 2 MiB huge mappings.
- Phase 3/3: "kernel code"

### Final Machine State
- BSP in 64-bit long mode
- All load segments of the kernel are loaded via 2 MiB huge page mapping with
  their corresponding page-table rights.
- APs are still asleep
- Register `%rdi` has pointer to %PROJECT_NAME% boot information
- `CR0` has the following bits set: PE (0), WP (1), PG (31)
- `CR3` holds the physical address of the root page table
- `CR4` has the following bits set: PSE (4), PAE (5)
- GDT living in the address space of the loader is set with one selector
  (64-bit, code segment, ring 0)

## Kernel Restrictions
Currently, the kernel you can load is not allowed to have relocation
information, as this is not supported yet. Furthermore, each load segment
needs to be aligned to a 2 MiB boundary for 2 MiB huge pages.

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

## Architecture
At first, my plan was to use only position-independent code

## Planned Functionality
- Loading ELFs with reloc information at a specified virtual address

//! Module for Xen PVH boot.

mod elf_note {
    use core::mem::size_of;

    // The PVH Boot Protocol starts at the 32-bit entrypoint to our firmware.
    extern "C" {
        fn entry_xenpvh();
    }

    /// Taken from <xen/include/public/elfnote.h>.
    const XEN_ELFNOTE_PHYS32_ENTRY: u32 = 18;
    type Name = [u8; 4];
    type Desc = unsafe extern "C" fn();

    #[repr(C, align(4))]
    struct ElfNote {
        name_size: u32,
        desc_size: u32,
        kind: u32,
        name: Name,
        desc: Desc,
    }

    #[link_section = ".note.xen_pvh"]
    #[used]
    static PVH_NOTE: ElfNote = ElfNote {
        name_size: size_of::<Name>() as u32,
        desc_size: size_of::<Desc>() as u32,
        kind: XEN_ELFNOTE_PHYS32_ENTRY,
        name: *b"Xen\0",
        desc: entry_xenpvh,
    };
}

//! Accessors to external symbols.

/// 2 MiB.
#[allow(non_upper_case_globals)]
const MiB2: u64 = 0x200000;

/// Definition of all external symbols. The data type must be an array for each
/// symbol, so the value of the symbol is the link address of this symbol.
/// Otherwise, it points to the value of that variable in the ELF symbols table.
///
/// So, either the type has to be en array or `core::ptr::addr_of!(SYMBOL)` is
/// required.
pub mod symbols {

    /// Modules from the boot code. These symbols are resolved by their link
    /// address in the low memory address space (see `LINK_ADDR_BOOT`).
    pub mod bootcode {
        extern "C" {
            /// L4 page table backing memory address in the link address space of
            /// the boot code.
            #[link_name = "boot_mem_page_table_l4"]
            pub static BOOT_MEM_PAGE_TABLE_L4: [u8; 0];

            /// L3 page table backing memory address in the link address space of
            /// the boot code.
            #[link_name = "boot_mem_page_table_l4"]
            pub static BOOT_MEM_PAGE_TABLE_L3: [u8; 0];
        }
    }

    /// Symbols from linker script and other sources.
    pub mod other {
        extern "C" {
            /// The link address of the boot code.
            #[link_name = "LINK_ADDR_BOOT"]
            pub static LINK_ADDR_BOOT: u64;

            /// The link address of the loader code.
            #[link_name = "LINK_ADDR_LOADER"]
            pub static LINK_ADDR_LOADER: u64;
        }
    }
}

/// Macro that takes the address of a symbol (in boot code link address space)
/// and transforms it to the loader link address space (1 G). This works
/// reliably as
/// - the build ensures that both address spaces never exceed 2 MiB
/// - both are in the same physical memory location
/// - the offset of the high addresses never clash with the offset of the low
///   addresses (see linker script)
macro_rules! symbol_in_loader_addr_space {
    ($symbol:path) => {
        {
            let ptr = unsafe { $symbol.as_ptr() };
            //let offset = (ptr as u64) & MiB2;
            //let addr = 0x40000000 + offset;
            //addr as *mut u64 as *mut u8
            ptr
        }
    };
}



pub fn link_addr_boot() -> u64 {
    (unsafe { core::ptr::addr_of!(symbols::other::LINK_ADDR_BOOT) }) as u64
}

pub fn link_addr_loader() -> u64 {
    (unsafe { symbols::other::LINK_ADDR_LOADER }) as u64
}

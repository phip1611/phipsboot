//! Definition of all external symbols. The data type must be an array for each
//! symbol, so the value of the symbol is the link address of this symbol.
//! Otherwise, it points to the value of that variable in the ELF symbols table.
//!
//! So, either the type has to be en array or `core::ptr::addr_of!(SYMBOL)` is
//! required.

pub use bootcode::*;
pub use other::*;

macro_rules! get_symbol_addr {
    ($ident:ident) => {
        (unsafe { $ident.as_ptr() }) as u64
    };
}

/// 2 MiB.
#[allow(non_upper_case_globals)]
const MiB2: u64 = 0x200000;

/// Modules from the boot code. These symbols are resolved by their link
/// address in the low memory address space (see `LINK_ADDR_BOOT`).
mod bootcode {
    use super::*;

    /// Boot code and loader code together fit within one single 2 MiB mapping.
    /// Both have different link addresses, but the offset of the high link
    /// address ensures that also the "low" symbol can be reached. This function
    /// performs this calculation and return the address of symbols from the
    /// boot code at the high link address.
    fn to_high_link_addr(addr: u64) -> u64 {
        let offset = addr & MiB2;
        link_addr_loader() + offset
    }

    extern "C" {
        /// L4 page table backing memory address in the link address space of
        /// the boot code.
        #[link_name = "boot_mem_page_table_l4"]
        static mut BOOT_MEM_PAGE_TABLE_L4: [u64; 0];

        /// L3 page table backing memory address in the link address space of
        /// the boot code.
        #[link_name = "boot_mem_page_table_l4"]
        static mut BOOT_MEM_PAGE_TABLE_L3: [u64; 0];
    }

    /// Returns the address of the L4 mem table in the high address space of the
    /// loader.
    pub fn boot_mem_page_table_l4() -> u64 {
        let addr = get_symbol_addr!(BOOT_MEM_PAGE_TABLE_L4);
        to_high_link_addr(addr)
    }

    /// Returns the address of the L3 mem table in the high address space of the
    /// loader.
    pub fn boot_mem_page_table_l3() -> u64 {
        let addr = get_symbol_addr!(BOOT_MEM_PAGE_TABLE_L3);
        to_high_link_addr(addr)
    }
}

/// Symbols from linker script and other sources.
mod other {
    use super::*;

    extern "C" {
        /// The link address of the boot code.
        #[link_name = "LINK_ADDR_BOOT"]
        static LINK_ADDR_BOOT: [u64; 1];

        /// The link address of the loader code.
        #[link_name = "LINK_ADDR_LOADER"]
        static LINK_ADDR_LOADER: [u64; 0];
    }

    /// Returns the 2-MiB aligned link address of the boot (assembly) code.
    pub fn link_addr_boot() -> u64 {
        get_symbol_addr!(LINK_ADDR_BOOT) & !MiB2
    }

    /// Returns the 2-MiB aligned link address of the loader (Rust) code.
    pub fn link_addr_loader() -> u64 {
        get_symbol_addr!(LINK_ADDR_LOADER) & !MiB2
    }
}

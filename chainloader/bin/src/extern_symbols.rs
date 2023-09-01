//! Definition of all external symbols. The data type must be an array for each
//! symbol, so the value of the symbol is the link address of this symbol.
//! Otherwise, it points to the value of that variable in the ELF symbols table.
//!
//! So, either the type has to be en array or `core::ptr::addr_of!(SYMBOL)` is
//! required.

pub use bootcode::*;
pub use other::*;

/// 2 MiB.
#[allow(non_upper_case_globals)]
const MiB2: u64 = 0x200000;

/// Modules from the boot code. These symbols are resolved by their link
/// address in the low memory address space (see `LINK_ADDR_BOOT`).
mod bootcode {
    extern "C" {
        /// L4 page table backing memory address in the link address space of
        /// the boot code.
        #[link_name = "boot_mem_page_table_l4"]
        static mut BOOT_MEM_PAGE_TABLE_L4: u64;

        /// L3 page table backing memory address in the link address space of
        /// the boot code.
        #[link_name = "boot_mem_page_table_l4"]
        static mut BOOT_MEM_PAGE_TABLE_L3: [u8; 0];
    }

    #[inline(never)]
    pub fn boot_mem_page_table_l4() -> u64 {
        (unsafe { core::ptr::addr_of!(BOOT_MEM_PAGE_TABLE_L4) }) as u64
    }

    /*/// Returns the address of the L4 page table in the loaders virtual
    /// address space.
    pub fn boot_mem_page_table_l4() -> *mut u8 {
        let link_addr_boot = unsafe { BOOT_MEM_PAGE_TABLE_L4.as_mut_ptr() };
    }*/
}

/// Symbols from linker script and other sources.
mod other {
    extern "C" {
        /// The link address of the boot code.
        #[link_name = "LINK_ADDR_BOOT"]
        static LINK_ADDR_BOOT: [u64; 1];

        /// The link address of the loader code.
        #[link_name = "LINK_ADDR_LOADER"]
        static LINK_ADDR_LOADER: u64;
    }

    /// Returns the link address of the boot (assembly) code.
    pub fn link_addr_boot() -> u64 {
        unsafe { LINK_ADDR_LOADER }
    }

    /// Returns the link address of the loader (Rust) code.
    pub fn link_addr_loader() -> u64 {
        unsafe { core::ptr::addr_of!(LINK_ADDR_LOADER) as u64 }
    }
}

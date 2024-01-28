//! Definition of all external symbols. The data type must be an array for each
//! symbol, so the value of the symbol is the link address of this symbol.
//! Otherwise, it points to the value of that variable in the ELF symbols table.
//!
//! So, either the type has to be en array or `core::ptr::addr_of!(SYMBOL)` is
//! required.


#[allow(unused)]
#[allow(unused)]
pub use {
    bootcode::*,
    other::*,
};

/// Trace-print all relevant symbols.
pub fn trace() {
    log::trace!("boot_mem_pt_l4    = {:?}", boot_mem_pt_l4());
    log::trace!("boot_mem_pt_l3_hi = {:?}", boot_mem_pt_l3_hi());
    log::trace!("boot_mem_pt_l3_lo = {:?}", boot_mem_pt_l3_lo());
    log::trace!("boot_mem_pt_l2_hi = {:?}", boot_mem_pt_l2_hi());
    log::trace!("boot_mem_pt_l2_lo = {:?}", boot_mem_pt_l2_lo());
    log::trace!("boot_mem_pt_l1_hi = {:?}", boot_mem_pt_l1_hi());
}

/// Symbols with their link address from the boot code.
#[allow(unused)]
mod bootcode {
    use super::*;

    extern "C" {
        #[link_name = "boot_mem_pt_l4"]
        static mut BOOT_MEM_PT_L4: [u64; 0];

        #[link_name = "boot_mem_pt_l3_lo"]
        static mut BOOT_MEM_PT_L3_LO: [u64; 0];

        #[link_name = "boot_mem_pt_l3_hi"]
        static mut BOOT_MEM_PT_L3_HI: [u64; 0];

        #[link_name = "boot_mem_pt_l2_lo"]
        static mut BOOT_MEM_PT_L2_LO: [u64; 0];

        #[link_name = "boot_mem_pt_l2_hi"]
        static mut BOOT_MEM_PT_L2_HI: [u64; 0];

        #[link_name = "boot_mem_pt_l1_hi"]
        static mut BOOT_MEM_PT_L1_HI: [u64; 0];
    }

    pub fn boot_mem_pt_l4() -> *const u8 {
        (unsafe { BOOT_MEM_PT_L4.as_ptr() }).cast()
    }

    pub fn boot_mem_pt_l3_lo() -> *const u8 {
        (unsafe { BOOT_MEM_PT_L3_LO.as_ptr() }).cast()
    }

    pub fn boot_mem_pt_l3_hi() -> *const u8 {
        (unsafe { BOOT_MEM_PT_L3_HI.as_ptr() }).cast()
    }

    pub fn boot_mem_pt_l2_lo() -> *const u8 {
        (unsafe { BOOT_MEM_PT_L2_LO.as_ptr() }).cast()
    }

    pub fn boot_mem_pt_l2_hi() -> *const u8 {
        (unsafe { BOOT_MEM_PT_L2_HI.as_ptr() }).cast()
    }

    pub fn boot_mem_pt_l1_hi() -> *const u8 {
        (unsafe { BOOT_MEM_PT_L1_HI.as_ptr() }).cast()
    }
}

/// Symbols from linker script and other sources.
#[allow(unused)]
mod other {
    use super::*;

    extern "C" {
        /// The link address of the boot code.
        #[link_name = "LINK_ADDR_BOOT"]
        static LINK_ADDR_BOOT: [u64; 0];

        /// The link address of the loader code.
        #[link_name = "LINK_ADDR_LOADER"]
        static LINK_ADDR_LOADER: [u64; 0];
    }


}

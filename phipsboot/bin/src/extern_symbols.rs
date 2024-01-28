//! Definition of all external symbols. The data type must be an array for each
//! symbol, so the value of the symbol is the link address of this symbol.
//! Otherwise, it points to the value of that variable in the ELF symbols table.
//!
//! So, either the type has to be en array or `core::ptr::addr_of!(SYMBOL)` is
//! required.

#[allow(unused)]
#[allow(unused)]
pub use {bootcode::*, other::*};

/// Translates the low link address of a symbol to its high link address.
///
/// Background: Because of the 2 MiB huge identity mapping of the low-level code
/// every symbol is in two address spaces. However, only the high address spaces
/// have proper access permissions.
pub fn boot_symbol_to_high_address(addr: *const u8) -> *const u8 {
    // calculate offset regarding the 2 MiB-aligned base. We know that the
    // whole binary is <= 2 MiB in size.
    let offset = addr as u64 & 0x2fffff;
    unsafe { link_addr_high_base().add(offset as usize) }
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
        #[link_name = "LINK_ADDR_BOOT"]
        static LINK_ADDR_BOOT: [u64; 0];

        #[link_name = "LINK_ADDR_HIGH_BASE"]
        static LINK_ADDR_HIGH_BASE: [u64; 0];

        #[link_name = "LINK_ADDR_RX"]
        static LINK_ADDR_RX: [u64; 0];

        #[link_name = "LINK_ADDR_RO"]
        static LINK_ADDR_RO: [u64; 0];

        #[link_name = "LINK_ADDR_RW"]
        static LINK_ADDR_RW: [u64; 0];
    }

    pub fn link_addr_boot() -> *const u8 {
        (unsafe { LINK_ADDR_BOOT.as_ptr() }).cast()
    }

    pub fn link_addr_high_base() -> *const u8 {
        (unsafe { LINK_ADDR_HIGH_BASE.as_ptr() }).cast()
    }

    pub fn link_addr_rx() -> *const u8 {
        (unsafe { LINK_ADDR_RX.as_ptr() }).cast()
    }

    pub fn link_addr_ro() -> *const u8 {
        (unsafe { LINK_ADDR_RO.as_ptr() }).cast()
    }

    pub fn link_addr_rw() -> *const u8 {
        (unsafe { LINK_ADDR_RW.as_ptr() }).cast()
    }
}

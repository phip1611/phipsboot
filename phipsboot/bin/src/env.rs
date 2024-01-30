//! Everything regarding the environment of the kernel.

use core::cell::OnceCell;
use lib::safe::Safe;

static BOOT_VARIANT: Safe<OnceCell<BootVariant>> = Safe::new(OnceCell::new());
static BOOT_INFO_PTR: Safe<OnceCell<u64>> = Safe::new(OnceCell::new());

#[derive(Debug)]
enum BootVariant {
    Multiboot1,
    Multiboot2,
    XenPvh,
}

pub fn init(bootloader_magic: u64, bootloader_info_ptr: u64) {
    let boot_variant;
    if bootloader_magic == multiboot2::MAGIC as u64 {
        boot_variant = BootVariant::Multiboot2;
    } else if bootloader_magic == 0x2badB002 {
        // TODO use constant from crate
        boot_variant = BootVariant::Multiboot1;
    } else if bootloader_magic == 0x336ec578 {
        boot_variant = BootVariant::XenPvh;
    } else {
        panic!(
            "Unknown boot loader magic! magic={:#x?}, info_ptr={:#x?}",
            bootloader_magic, bootloader_info_ptr
        );
    }
    BOOT_VARIANT.get_or_init(|| boot_variant);
    BOOT_INFO_PTR.get_or_init(|| bootloader_info_ptr);
}

/// Trace-print all relevant symbols.
#[rustfmt::skip]
pub fn print() {
    log::debug!("PhipsBoot was loaded via   {:?}", BOOT_VARIANT.get().unwrap());
    log::debug!("              boot info at {:#016x} (phys)", BOOT_INFO_PTR.get().unwrap());
    log::debug!("          expected load at {:#016x} (phys)", crate::extern_symbols::link_addr_boot() as u64);
    log::debug!("            actual load at {:#016x} (phys)", load_addr());
    log::debug!("              with offset {}{:#x}",
        if crate::mem::load_offset() < 0 {
            "-"
        } else {
            " "
        },
        // Always print the positive value; we already added the sign.
        // Otherwise, negative values are printed as 0xfff...
        crate::mem::load_offset().abs()
    );

    trace_external_symbols();
}

/// Returns the physical address at which PhipsBoot was loaded.
fn load_addr() -> u64 {
    crate::mem::virt_to_phys(crate::extern_symbols::link_addr_boot().into()).into()
}

fn trace_external_symbols() {
    use crate::extern_symbols::*;

    log::trace!("");
    log::trace!("SYMBOL            |       VIRT (low) |        VIRT (high) |             PHYS");

    fn trace_boot_symbol(name: &str, symbol: *const u8) {
        log::trace!(
            "{name:<17} | {:016x?} | {:016x?} | {:#016x?} ",
            symbol,
            boot_symbol_to_high_address(symbol),
            crate::mem::virt_to_phys((symbol as u64).into()).val()
        );
    }
    trace_boot_symbol("boot_mem_pt_l4", boot_mem_pt_l4());
    trace_boot_symbol("boot_mem_pt_l3_hi", boot_mem_pt_l3_hi());
    trace_boot_symbol("boot_mem_pt_l3_lo", boot_mem_pt_l3_lo());
    trace_boot_symbol("boot_mem_pt_l2_hi", boot_mem_pt_l2_hi());
    trace_boot_symbol("boot_mem_pt_l2_lo", boot_mem_pt_l2_lo());
    trace_boot_symbol("boot_mem_pt_l1_hi", boot_mem_pt_l1_hi());
}

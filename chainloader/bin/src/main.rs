#![no_main]
#![no_std]

// #![feature(error_in_core)]

// extern crate alloc;

extern crate alloc;

mod asm;
mod extern_symbols;
mod mem;
mod driver;

use alloc::string::String;
use alloc::{format, vec};
use alloc::vec::Vec;
use core::fmt::Write;
use core::hint::black_box;
use core::panic::PanicInfo;
use core::str::FromStr;
use lib::logger;
use crate::mem::stack;

/// Entry into the high-level code of the loader.
///
/// # Machine State
/// - 64-bit long mode with 4-level paging
/// - `CR0` has the following bits set: PE (0), WP (1), PG (31)
/// - `CR3` holds the physical address of the root page table
/// - `CR4` has the following bits set: PAE (5)
///
/// # Paging
/// The hole loader is reachable via its link address (2 MiB mapping) and via
/// an identity mapping of the physical location in memory.
#[no_mangle]
extern "C" fn rust_entry(
    bootloader_magic: u64,
    bootloader_info_ptr: u64,
    load_addr_offset: i64,
) -> ! {
    mem::init(load_addr_offset);
    logger::init(); // logger depends on an enabled heap

    logger::add_backend(driver::DebugconLogger::default()).unwrap();
    logger::flush(); // flush all buffered messages

    let vec = vec![1, 2, 3];

    log::info!("AFTER logger init {vec:#x?}");
    log::debug!("magic               = {:#x?}", bootloader_magic);
    log::debug!("bootloader_info_ptr = {:#x?}", bootloader_info_ptr);
    log::debug!("load_addr_offset    = {:#x?}", load_addr_offset);
    stack::assert_sanity_checks();
    break_stack();
    loop {}
}

#[inline(never)]
fn break_stack() {
    stack::assert_sanity_checks();
    log::debug!("stack usage: {:#.2?}", stack::usage());
    break_stack();
}


#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    // If a panic happens, we are screwed anyways. We do some additional
    // emergency logging without the whole log-stack
    let _ = writeln!(&mut driver::DebugconLogger, "PANIC: {info:#?}");

    // log::error!("PANIC: {info:#?}");

    unsafe {
        // TODO only do this when no logging is initialized?!
        core::arch::asm!("ud2", in("rax") 0xbadb001);
    }
    loop {}
}

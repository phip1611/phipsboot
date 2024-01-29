#![feature(abi_x86_interrupt)]
#![no_main]
#![no_std]

// #![feature(error_in_core)]

// extern crate alloc;

extern crate alloc;

mod asm;
mod driver;
mod env;
mod extern_symbols;
mod idt;
mod mem;
mod xen_pvh;

use crate::mem::stack;
use core::fmt::Write;
use core::hint::black_box;
use core::panic::PanicInfo;
use lib::logger;

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
extern "C" fn rust_entry64(
    bootloader_magic: u64,
    bootloader_info_ptr: u64,
    load_addr_offset: i64,
) -> ! {
    // The order of the init functions mostly reflect actual dependencies!
    idt::init();
    mem::init(load_addr_offset);
    logger::init(); // after mem init; logger depends on heap!
    logger::add_backend(driver::DebugconLogger::default()).unwrap();
    logger::flush(); // flush all buffered messages

    env::init(bootloader_magic, bootloader_info_ptr);
    env::print();

    stack::assert_sanity_checks();

    log::info!("Now loading your kernel into 64-bit mode...");
    log::info!("Not implemented yet! =(");

    // break_stack();
    create_pagefault();

    loop {}
}

/// Sometimes useful to test the stack + stack canary.
#[allow(unused, unconditional_recursion)]
#[inline(never)]
fn break_stack() {
    log::debug!("Breaking stack ...");
    stack::assert_sanity_checks();
    log::debug!("stack usage: {:#.2?}", stack::usage());
    break_stack();
}

/// Sometimes useful to test the binary.
#[allow(unused)]
fn create_pagefault() {
    log::debug!("Creating page fault ...");
    let ptr = core::ptr::null::<u8>();
    unsafe {
        black_box(core::ptr::read_volatile(ptr));
    }
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

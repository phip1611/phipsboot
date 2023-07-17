#![no_main]
#![no_std]
#![feature(error_in_core)]

// extern crate alloc;

mod asm;
mod stack;
pub(crate) mod extern_symbols;
mod mem;

pub(crate) use crate::debugcon::Printer;
use core::fmt::Write;
use core::panic::PanicInfo;
use core::str::FromStr;

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
    multiboot2_magic: u64,
    multiboot2_ptr: u64,
    load_addr_offset: u64,
) -> ! {
    init(multiboot2_magic, multiboot2_ptr, load_addr_offset);
    let _ = Printer.write_str("Hello World from Rust Entry\n");
    let _ = writeln!(Printer, "magic: {:#x?}, ptr: {:#x?}, load_addr_offset: {:#x?}", multiboot2_magic, multiboot2_ptr, load_addr_offset);
    let _ = writeln!(Printer, "stack_begin: {:#?}, end_begin: {:#?}", extern_symbols::stack_begin(), extern_symbols::stack_end());

    // let x = lib::cli::CliArgs::from_str("").unwrap();
    //let _x = core::hint::black_box(x);

    // stack::assert_canary(load_addr_offset);

    // break_stack(load_addr_offset);

    loop {}
}

fn break_stack(load_addr_offset: u64) {
    stack::assert_canary(load_addr_offset);
    break_stack(load_addr_offset);
}

fn init(
    multiboot2_magic: u64,
    multiboot2_ptr: u64,
    load_addr_offset: u64,) {
    mem::init(load_addr_offset);
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    let _ = writeln!(Printer, "PANIC: {info:#?}");
    loop {}
}

#[inline(never)]
fn a(x: u8) -> u8 {
    b(core::hint::black_box(x)) + 1
}

#[inline(never)]
fn b(x: u8) -> u8 {
    c(core::hint::black_box(x)) + 2
}

#[inline(never)]
fn c(x: u8) -> u8 {
    core::hint::black_box(x) + 3
}

// Transforms a number to a hex strin with leading "0x" and 8 hex digits.
fn u32_to_hex_string_in_buf(mut num: u32, buffer: &mut [u8; 10]) -> &str {
    // reset buffer
    for i in 0..10 {
        buffer[i] = 0;
    }

    for (i, _) in (0..32 / 4).enumerate() {
        let hex_digit = (num & 0xf) as u8;
        buffer[buffer.len() - 1 - i] = if hex_digit < 10 {
            b'0' + hex_digit
        } else {
            b'a' + (hex_digit - 10)
        };
        num >>= 4;
    }

    buffer[0] = b'0';
    buffer[1] = b'x';

    core::str::from_utf8(buffer).unwrap()
}

mod debugcon {
    use super::*;

    const QEMU_DEBUGCON_PORT: u16 = 0xe9;

    pub struct Printer;

    impl core::fmt::Write for Printer {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            for byte in s.as_bytes() {
                print_char(*byte);
            }
            Ok(())
        }
    }

    fn print_char(c: u8) {
        unsafe { x86::io::outb(QEMU_DEBUGCON_PORT, c) }
    }
}

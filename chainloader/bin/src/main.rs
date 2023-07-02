#![no_main]
#![no_std]
#![feature(error_in_core)]

// extern crate alloc;

mod asm;

use crate::debugcon::Printer;
use core::fmt::Write;
use core::panic::PanicInfo;

#[no_mangle]
extern "C" fn rust_entry(
    multiboot2_magic: u32,
    multiboot2_information_ptr: u32,
    load_addr_offset: u32,
) -> ! {
    let _ = Printer.write_str("Hello World from Rust Entry\n");
    // I manually verified that all of the above values are passed correctly.
    // Even in the relocated case.

    // The next test case verifies that the stack works:

    // 0 + 1 + 2 + 3 should equal 6
    let sum_via_stacked_function_calls = a(0);
    let _ = Printer.write_str("0 + 1 + 2 + 3 = ");
    unsafe { x86::io::outb(0xe9, b'0' + sum_via_stacked_function_calls) }
    let _ = Printer.write_str("\n");

    // This text case verifies that .rodata symbols can be read even when things
    // were relocated.

    let val = unsafe { core::ptr::read_volatile("rodata symbols can be read".as_ptr()) };
    assert_eq!(val, b'r');

    // Output some helpful stuff.
    // This works as non of the code using formatting magic.
    {
        // print MB2 Magic
        let _ = Printer.write_str("\nMBI magic: ");
        let mut buf = [0; 10];
        let str_to_print = u32_to_hex_string_in_buf(multiboot2_magic, &mut buf);
        let _ = Printer.write_str(str_to_print);

        let _ = Printer.write_str("\nMBI pointer: ");
        let mut buf = [0; 10];
        let str_to_print = u32_to_hex_string_in_buf(multiboot2_information_ptr, &mut buf);
        let _ = Printer.write_str(str_to_print);

        let _ = Printer.write_str("\nLoad Offset: ");
        let mut buf = [0; 10];
        let str_to_print = u32_to_hex_string_in_buf(load_addr_offset, &mut buf);
        let _ = Printer.write_str(str_to_print);

        let _ = Printer.write_str("\nExpected Runtime Address (start symbol): ");
        let mut buf = [0; 10];
        /*extern "C" {
            #[link_name = "start"]
            static mut START_SYMBOL: u32;
        }
        //let start_symbol_addr = unsafe { core::ptr::addr_of!(START_SYM) } as u32;
        // let start_symbol_addr = unsafe { &START_SYMBOL as *const u32 } as u32;*/
        let start_symbol_addr = 0x200080; // taken from objdump
        let str_to_print = u32_to_hex_string_in_buf(start_symbol_addr, &mut buf);
        let _ = Printer.write_str(str_to_print);

        let _ = Printer.write_str("\nReal Runtime Address (start symbol): ");
        let addr = start_symbol_addr + load_addr_offset;
        let mut buf = [0; 10];
        let str_to_print = u32_to_hex_string_in_buf(addr, &mut buf);
        let _ = Printer.write_str(str_to_print);

        let _ = Printer.write_str("\n");
    }


    // Now, this here breaks everything and starts a weird loop as something
    // statically jumps back to the 2MiB address range (should be 6 Mib because
    // of the relocation)


    // Helper to find the problematic code in assembly.
    unsafe {
        core::arch::asm!("", in("eax") 0xdeadbeef_u32);
    }

    // TODO I have to figure out why this breaks everything.
    let x = format_args!("hello");
    let _ = Printer.write_fmt(x);

    loop {}
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

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
    unsafe {
        core::arch::asm!(
            "",
            in("eax") 0xdeadbeef_u32,
            in("ebx") multiboot2_magic,
            in("ecx") multiboot2_information_ptr,
            in("edx") load_addr_offset,
            options(att_syntax)
        );
    }

    // 0 + 1 + 2 + 3 should equal 6
    let sum_via_stacked_function_calls = a(0);
    Printer.write_str("0 + 1 + 2 + 3 = ");
    unsafe { x86::io::outb(0xe9, b'0' + sum_via_stacked_function_calls) }
    Printer.write_str("\n");


    let val = unsafe { core::ptr::read_volatile("foo".as_ptr()) };
    assert_eq!(val, b'f');

    // print MB2 Magic
    Printer.write_str("\nMBI magic: ");
    let mut buf = [0; 10];
    let str_to_print = u32_to_hex_string_in_buf(multiboot2_magic, &mut buf);
    Printer.write_str(str_to_print);

    Printer.write_str("\nMBI pointer: ");
    let mut buf = [0; 10];
    let str_to_print = u32_to_hex_string_in_buf(multiboot2_information_ptr, &mut buf);
    Printer.write_str(str_to_print);

    Printer.write_str("\nLoad Offset: ");
    let mut buf = [0; 10];
    let str_to_print = u32_to_hex_string_in_buf(load_addr_offset, &mut buf);
    Printer.write_str(str_to_print);
    Printer.write_str("\n");

    // TODO I have to figure out why this breaks everything.
    let x = format_args!("hello");
    Printer.write_fmt(x);

    loop {}

    loop {}

    let x = format_args!("hello");
    unsafe { x86::io::outb(0xe9, b'\n') };
    Printer.write_fmt(x);
    writeln!(Printer, "hello world").unwrap();
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

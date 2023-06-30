#![no_main]
#![no_std]
#![feature(error_in_core)]

// extern crate alloc;

use core::panic::PanicInfo;
use crate::debugcon::Printer;
use core::fmt::Write;

core::arch::global_asm!(include_str!("asm/start.S"), options(att_syntax));
core::arch::global_asm!(include_str!("asm/multiboot2_header.S"), options(att_syntax));

#[no_mangle]
extern "C" fn rust_entry(
    multiboot2_magic: u32,
    multiboot2_information_ptr: u32,
    load_addr_offset: u32,
) -> ! {
    writeln!(Printer, "hello world").unwrap();
    loop {}
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    let _ = writeln!(Printer, "PANIC");
    loop {}
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

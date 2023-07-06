use core::fmt::{Debug, Write};
use crate::debugcon::Printer;
use crate::extern_symbols;

const CANARY: u32 = 0xdeadbeef;

pub struct StackDescriptor {
    pub virt_addr: u64,
    pub phys_addr: u64,
}

pub fn sanity_checks() {

}

/// Assert there was no stack-overflow yet by checking a fixed canary value at
/// the end of the stack.
pub fn assert_canary(load_addr_offset: u64) {
    unsafe {
        let _ = writeln!(Printer, "STACK_BEGIN = {:#x?}, STACK_END = {:#x?}, LEN = {:#x?}", extern_symbols::stack_begin_link_addr().add(load_addr_offset as usize), extern_symbols::stack_end_link_addr().add(load_addr_offset as usize), len());
    }

    let slice = unsafe { core::slice::from_raw_parts(extern_symbols::stack_begin_link_addr().add(load_addr_offset as usize).cast::<u32>(), len() / 4) };
    for (i, &byte) in slice.iter().enumerate() {
        if byte != 0 {
            let _ = writeln!(Printer, "stack != 0 at {:x?} => {:x?}", i, byte);
        }
    }

    assert_eq!(CANARY, canary(load_addr_offset));
}

fn len() -> usize {
    let begin = extern_symbols::stack_begin_link_addr() as usize;
    let end = extern_symbols::stack_end_link_addr() as usize;
    assert!(end > begin);
    end - begin
}

fn canary(load_addr_offset: u64) -> u32 {
    let addr = unsafe { extern_symbols::stack_begin_link_addr().add(load_addr_offset as usize) };
    let addr = addr.cast::<u32>();
    *unsafe { &*addr }
}

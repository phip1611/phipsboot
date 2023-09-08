use crate::debugcon::Printer;
use crate::extern_symbols;
use core::fmt::{Debug, Write};
use core::ops::{Deref, DerefMut};
use lib::mem::stack::{DEFAULT_STACK_SIZE, Stack};
use lib::safe::Safe;

/// Backing memory for the stack. This is mutable and lands in the `.data`
/// section.
static mut STACK: Stack<DEFAULT_STACK_SIZE> = Stack::new();

/// Aligned ready-to-use top of the stack.
#[no_mangle]
static STACK_TOP: Safe<*const u8> = Safe::new(unsafe { STACK.adjusted_top() });


/// Initializes the stack.
pub fn init() {

}

/// Sanity checks for the stack. Verifies:
/// - canary
/// - stack pointer not out of bounds
pub fn assert_sanity_checks() {
    unsafe { assert_eq!(Ok(()), STACK.check_canary()); }
    let current_rsp: u64;
    unsafe { core::arch::asm!("mov %rsp, %rax", out("rax") current_rsp, options(att_syntax)) };
    assert!(current_rsp < unsafe { STACK.top() } as u64);
    assert!(current_rsp >= unsafe { STACK.bottom() } as u64);
}

/// Returns the aligned ready-to-use top of the stack.
pub fn top() -> *mut u8 {
    unsafe { STACK.adjusted_top() }
}


/// Returns the usable size of the stack.
pub fn usable_size() -> u64 {
    top() as u64 - bottom() as u64
}

/// Returns the inclusive bottom of the stack.
pub fn bottom() -> *mut u8 {
    unsafe { STACK.bottom() }
}

/// Returns the current stack usage in bytes.
#[inline(never)]
pub fn usage() -> u64 {
    let current_rsp: u64;
    unsafe { core::arch::asm!("mov %rsp, %rax", out("rax") current_rsp, options(att_syntax)) };
    top() as u64 - current_rsp
}

/// Returns the current canary.
pub fn canary() -> u64 {
    unsafe { STACK.canary() }
}

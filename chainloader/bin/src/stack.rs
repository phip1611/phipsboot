use crate::debugcon::Printer;
use crate::extern_symbols;
use core::fmt::{Debug, Write};
use core::ops::{Deref, DerefMut};
use lib::safe::Safe;

/// Size of the stack in bytes.
pub const SIZE: usize = 0x10000 /* 64 KiB */;

/// Canary value at the bottom of the stack to detect overflows.
const CANARY: u64 = 0x13371337_deadbeef;

/// Backing memory for the stack. This is mutable and lands in the `.data`
/// section.
#[link_section = ".data"] // ensure this is r/w and not r/o.
static STACK: Stack<SIZE> = Stack::new();

/// Exclusive address of the stack top.
#[used]
#[no_mangle]
static STACK_TOP: Safe<*const u8> = Safe::new(STACK.top());

/// Properly aligned type holding backing memory for stack.
#[repr(C, align(16))]
struct Stack<const N: usize>([u8; N]);

impl<const Size: usize> Stack<Size> {
    const fn new() -> Self {
        Self([0; Size])
    }

    const fn top(&self) -> *mut u8 {
        unsafe { self.bottom().add(Size) }
    }

    const fn bottom(&self) -> *mut u8 {
        self.0.as_ptr().cast_mut()
    }
}

/// Initializes the stack.
pub fn init() {
    write_canary();
}

/// Sanity checks for the stack. Verifies:
/// - canary
/// - stack pointer not out of bounds
pub fn sanity_checks() {
    assert_eq!(current_canary(), CANARY);
    let current_rsp: u64;
    unsafe {
        core::arch::asm!("mov %rsp, %rax", out("rax") current_rsp, options(att_syntax))
    };
    assert!(current_rsp < STACK.top() as u64);
    assert!(current_rsp >= STACK.bottom() as u64);
}

/// Returns the exclusive top of the stack.
pub fn top() -> *const u8 {
    STACK.top()
}

/// Returns the exclusive bottom of the stack.
pub fn bottom() -> *const u8 {
    STACK.bottom()
}

/// Returns the current stack usage in bytes.
#[inline(never)]
pub fn current_usage() -> u64 {
    let current_rsp: u64;
    unsafe {
        core::arch::asm!("mov %rsp, %rax", out("rax") current_rsp, options(att_syntax))
    };
    STACK.top() as u64 - current_rsp
}

/// Writes the canary value of the stack. This should only be done once during
/// initialization.
fn write_canary() {
    // volatile: needed as STACK is not "mut" for simplicity.
    unsafe { core::ptr::write_volatile(canary_address(), CANARY); }
}

/// Returns the address of the canary at the bottom of the stack.
fn canary_address() -> *mut u64 {
    STACK.bottom().cast::<u64>()
}

pub fn current_canary() -> u64 {
    // volatile: needed as STACK is not "mut" for simplicity.
    unsafe { core::ptr::read_volatile(canary_address()) }
}

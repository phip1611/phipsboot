use core::fmt::{Debug};
use lib::mem::stack::{Stack, DEFAULT_STACK_SIZE};
use lib::safe::Safe;

/// Backing memory for the stack. This is mutable and lands in the `.data`
/// section.
#[no_mangle] // Useful to find the stack location with readelf.
static mut STACK: Stack<DEFAULT_STACK_SIZE> = Stack::new();

/// Symbol that holds the pointer to the actual ready-to-use top of the stack.
/// This gives the assembly code access to the correct link address.
#[no_mangle]
static STACK_TOP_PTR: Safe<*const u8> = Safe::new(unsafe { STACK.adjusted_top() });

/// The stack is already initialized in the assembly routine. Otherwise, we
/// wouldn't get here. We just do some sanity checks and log debug information
/// about the stack.
pub fn init() {
    assert_sanity_checks();
}

/// Sanity checks for the stack. Verifies:
/// - canary
/// - stack pointer not out of bounds
#[inline(never)]
pub fn assert_sanity_checks() {
    unsafe {
        assert_eq!(Ok(()), STACK.check_canary());
    }
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

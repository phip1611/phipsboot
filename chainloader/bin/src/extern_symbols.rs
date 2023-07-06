
extern "C" {
    #[link_name = "stack_begin"]
    static STACK_BEGIN: [u8; 0];
    #[link_name = "stack_end"]
    static STACK_END: [u8; 0];
}

/// Inclusive begin of the stack.
pub fn stack_begin_link_addr() -> *const u8 {unsafe {
    core::ptr::addr_of!(STACK_BEGIN).cast()}
}

/// Exclusive end of the stack.
pub fn stack_end_link_addr() -> *const u8 {unsafe {
    core::ptr::addr_of!(STACK_END).cast()}
}

//! Accessors to external symbols. All external symbols in the Rust code

/// All external symbols. The data type must be an array for each symbol, so
/// that tha value of the symbol is the link address of that symbol. Otherwise,
/// it points to the value of that variable in the ELF symbols table
/// (TODO, is this correct?).
/// So, either the type has to be en array or `core::ptr::addr_of!(SYMBOL)` is
/// required.
mod symbols {
    extern "C" {
        #[link_name = "stack_begin"]
        pub static STACK_BEGIN: [u8; 0];
        #[link_name = "stack_end"]
        pub static STACK_END: [u8; 0];
    }
}

/// Inclusive begin of the stack.
pub fn stack_begin() -> *const u8 {
    unsafe { symbols::STACK_BEGIN.as_ptr() }
}

/// Exclusive end of the stack.
pub fn stack_end() -> *const u8 {
    unsafe { symbols::STACK_END.as_ptr() }
}

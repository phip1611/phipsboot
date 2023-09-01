//! Abstraction for managing memory of the system and the loader.


use core::cell::OnceCell;
use lib::safe::Safe;
use crate::stack;

pub type LoadOffsetT = i64;

pub static ONCE: Safe<OnceCell<u64>> = Safe::new(OnceCell::new());

pub fn init(load_offset: u64) {
    let _ = ONCE.get_or_init(|| load_offset);
    stack::init();
    init_heap();
}
/*

static mut HEAP: [u8; 4096] = [0; 4096];
*/
#[global_allocator]
static ALLOC: good_memory_allocator::SpinLockedAllocator = good_memory_allocator::SpinLockedAllocator::empty();


fn init_heap() {
    // unsafe { ALLOC.init(HEAP.as_ptr() as usize, HEAP.len()); }
}

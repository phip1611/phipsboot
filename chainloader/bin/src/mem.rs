//! Abstraction for managing memory of the system and the loader.

use lib::once::Once;

pub type LoadOffsetT = i64;

pub static ONCE: Once<u64> = Once::new();

pub fn init(load_offset: u64) {
    init_heap();
}

static mut HEAP: [u8; 4096] = [0; 4096];

#[global_allocator]
static ALLOC: good_memory_allocator::SpinLockedAllocator = good_memory_allocator::SpinLockedAllocator::empty();

fn init_heap() {
    unsafe { ALLOC.init(HEAP.as_ptr() as usize, HEAP.len()); }
}

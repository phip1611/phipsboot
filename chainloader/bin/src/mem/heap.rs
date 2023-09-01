//! Abstraction for managing memory of the system and the loader.


pub(super) fn init() {
}
/*

static mut HEAP: [u8; 4096] = [0; 4096];
*/
#[global_allocator]
static ALLOC: good_memory_allocator::SpinLockedAllocator = good_memory_allocator::SpinLockedAllocator::empty();



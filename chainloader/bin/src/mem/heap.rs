//! Abstraction for managing memory of the system and the loader.

/// Size of the heap.
const SIZE: usize = 0x20000 /* 128 KiB */;

/// Backing memory for the heap.
static mut HEAP: [u8; SIZE] = [0; SIZE];

#[global_allocator]
static ALLOC: good_memory_allocator::SpinLockedAllocator =
    good_memory_allocator::SpinLockedAllocator::empty();

pub fn init() {
    unsafe { ALLOC.init(HEAP.as_ptr() as usize, SIZE) }
}

//! Abstraction for managing memory of the system and the loader.

use core::cell::OnceCell;
use lib::mem::paging::{PhysAddr, VirtAddr};
use lib::safe::Safe;

mod heap;
pub mod stack;

/// Stores the load offset of the loader in physical memory.
static ONCE: Safe<OnceCell<i64>> = Safe::new(OnceCell::new());

pub fn init(load_offset: i64) {
    let _ = ONCE.get_or_init(|| load_offset);
    stack::init();
    heap::init();
}

/// Returns the load offset of the loader in physical memory.
pub fn load_offset() -> i64 {
    *ONCE.get().expect("should have been configured")
}

/// Translates the virtual link address to a physical address in memory.
pub fn virt_to_phys(virt: VirtAddr) -> PhysAddr {
    // assert_eq!(virt >= );
    (virt.val() as i64 + load_offset()).into()
}

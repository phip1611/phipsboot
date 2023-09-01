//! Types and helpers for x86_64 4-level paging.

pub const PAGE_TABLE_ENTRY_SIZE: u64 = core::mem::size_of::<u64>() as u64;

/// 9 bits select the entry of the given page table.
pub const INDEX_BITMASK: u64 = 0x1ff;

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Hash, Eq, Ord)]
pub enum Level {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
}

impl Level {
    pub fn val(self) -> u64 {
        self as u64
    }
}

/// Helper for common impls of phys and virt addresses.
macro_rules! impl_addr {
    ($typ:ty) => {
        impl $typ {
            /// Constructor.
            pub fn new(val: u64) -> Self {
                Self(val)
            }

            /// Returns the inner value.
            pub fn val(self) -> u64 {
                self.0
            }
        }

        impl From<u64> for $typ {
            fn from(val: u64) -> Self {
                Self::new(val)
            }
        }

        impl From<$typ> for u64 {
            fn from(val: $typ) -> Self {
                val.0
            }
        }
    };
}

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Hash, Eq, Ord, Default)]
#[repr(transparent)]
pub struct VirtAddr(u64);

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Hash, Eq, Ord, Default)]
#[repr(transparent)]
pub struct PhysAddr(u64);

impl_addr!(PhysAddr);
impl_addr!(VirtAddr);

impl VirtAddr {
    /// Returns the index into the page table of the given level.
    /// The returned value is in range `0..512`.
    pub fn pt_index(&self, level: Level) -> u64 {
        let level = level.val();
        let bits = self.val() >> ((level - 1) * 9) + 12;
        bits & INDEX_BITMASK
    }

    /// Returns the byte offset into the page table of the given level.
    /// The returned value is in range `0..4096`.
    pub fn pt_offset(&self, level: Level) -> u64 {
        self.pt_index(level) * PAGE_TABLE_ENTRY_SIZE
    }
}

/// Creates one single 1 GiB mapping with rwx permissions.
pub fn map_1g_rwx(src: VirtAddr, dest: PhysAddr, flags: u64) {

}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that the indices and offsets into page tables are properly
    /// calculated. I used the "paging-calculator" facility to verify those
    /// results.
    #[test]
    fn page_table_index_and_offset() {
        let addr = VirtAddr::from(0xdead_beef_1337_1337);
        assert_eq!(addr.pt_index(Level::One), 369);
        assert_eq!(addr.pt_index(Level::Two), 153);
        assert_eq!(addr.pt_index(Level::Three), 444);
        assert_eq!(addr.pt_index(Level::Four), 381);
        assert_eq!(addr.pt_offset(Level::One), 0xb88);
        assert_eq!(addr.pt_offset(Level::Two), 0x4c8);
        assert_eq!(addr.pt_offset(Level::Three), 0xde0);
        assert_eq!(addr.pt_offset(Level::Four), 0xbe8);
    }
}

//! Abstraction for managing memory of the system and the loader.

use lib::once::Once;

pub type LoadOffsetT = i64;

pub static ONCE: Once<u64> = Once::new();

pub fn init(load_offset: u64) {}

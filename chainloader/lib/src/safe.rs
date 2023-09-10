//! Module for [`Safe`].

use core::ops::{Deref};

/// Smart pointer that marks the inner type as [`Send`] and [`Sync`]. This is
/// useful as wrapper for global statics, if the inner type alone is usually not
/// allowed to be shared globally in standard Rust.
///
/// This wrapper is safe in context of this loader, as there are no concurrent
/// threads or interrupt handling, but just single-core execution.
pub struct Safe<T>(T);

impl<T> Safe<T> {
    pub const fn new(t: T) -> Self {
        Safe(t)
    }
}

impl<T> Deref for Safe<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl<T> Send for Safe<T> {}
unsafe impl<T> Sync for Safe<T> {}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        static FOO: Safe<*const u8> = Safe::new(0xdeadbeef_u32 as _);
        assert_eq!(*FOO as u64, 0xdeadbeef);
    }
}

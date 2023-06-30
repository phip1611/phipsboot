//! Module for [`Safe`].

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

/// Smart pointer that marks the inner type as [`Send`] and [`Sync`]. This is
/// safe for all usage in the context of this loader as it don't use SMP,
/// interrupts, or any other form of concurrency.
///
/// TODO: currently, this type allows multiple mutable references to the same
///  underlying type!
#[derive(Debug)]
pub struct Safe<T>(UnsafeCell<T>);

impl<T> Safe<T> {
    /// Constructor.
    pub const fn new(value: T) -> Self {
        Self(UnsafeCell::new(value))
    }
}

unsafe impl<T> Send for Safe<T> {}
unsafe impl<T> Sync for Safe<T> {}

impl<T> Deref for Safe<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.get() }
    }
}

impl<T> DerefMut for Safe<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
       unsafe { &mut *self.0.get() }
    }
}

/*struct SafeRef<'a> {

} */

#[cfg(test)]
mod tests {
    use alloc::string::String;
    use super::*;

    #[test]
    fn test_safe() {
        let mut foo = Safe::new(1337);
        let read: &i32 = &foo;
        assert_eq!(*read, 1337);
        let write: &mut i32 = &mut foo;
        *write = 0;
        let read: &i32 = &foo;
        assert_eq!(*read, 0);
    }
}

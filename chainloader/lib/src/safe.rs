//! Module for [`Safe`].

use core::ops::{Deref, DerefMut};
use std::cell::RefCell;

/// Smart pointer around [`RefCell`] that is [`Send`] and [`Sync`]. This type is
/// safe for all usage in the context of this loader as the loader don't use
/// SMP, interrupts, or any other form of concurrency.
///
/// It is not suited for `const` accesses, as it performs borrow checks during
/// runtime.
#[derive(Debug)]
pub struct Safe<T>(RefCell<T>);

impl<T> Safe<T> {
    /// Constructor.
    pub const fn new(value: T) -> Self {
        Self(RefCell::new(value))
    }
}

impl<T: Default> Default for Safe<T> {
    fn default() -> Self {
        Safe::new(T::default())
    }
}

unsafe impl<T> Send for Safe<T> {}
unsafe impl<T> Sync for Safe<T> {}

impl<T> Deref for Safe<T> {
    type Target = RefCell<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Safe<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
       &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let foo = Safe::new(1337);
        let read = *foo.borrow();
        assert_eq!(read, 1337);
        let mut write = foo.borrow_mut();
        *write = 42;
        assert_eq!(*write, 42);
    }

    #[test]
    #[should_panic]
    fn test_panic_multiple_writer() {
        let foo = Safe::new(1337);
        let _w1 = foo.borrow_mut();
        let _w2 = foo.borrow_mut();
    }

    #[test]
    #[should_panic]
    fn test_panic_writer_and_reader() {
        let foo = Safe::new(1337);
        let _w = foo.borrow_mut();
        let _r = foo.borrow();
    }
}

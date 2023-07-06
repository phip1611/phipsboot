use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

/// A smart pointer for a eventually manually initialized value that is `Send`
/// and `Sync`. This is fine in the loader, as there is no interrupt handling or
/// SMP.
///
/// Must be initialized with [`Once::init`]. Otherwise, dereferencing the smart
/// pointer panics.
#[derive(Debug)]
pub struct Once<T>(UnsafeCell<Option<T>>);

impl<T> Once<T> {
    /// Constructor.
    pub const fn new() -> Self {
        Self(UnsafeCell::new(None))
    }

    /// Initializes the value. Can be called exactly once, otherwise panics.
    pub fn init(&self, val: T) {
        let inner = unsafe { &mut * self.0.get() };
        let old = inner.replace(val);
        if old.is_some() {
            panic!("should not be initialized");
        }
    }
}

// Safety: I don't use SMP or interrupts in the loader.
unsafe impl<T> Send for Once<T> {}
unsafe impl<T> Sync for Once<T> {}

impl<T> Deref for Once<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let option = unsafe { &*self.0.get() }.as_ref();
        option.expect("should be initialized")
    }
}

impl<T> DerefMut for Once<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let option = unsafe { &mut *self.0.get() }.as_mut();
        option.expect("should be initialized")
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;
    use super::*;

    #[test]
    fn test_once_smart_pointer() {
        let foo = Once::<String>::new();
        foo.init(String::from("hello"))
    }

    #[test]
    fn test_once_smart_pointer_static() {
        static FOO: Once<String> = Once::new();
        FOO.init(String::from("hello"))
    }

    #[test]
    #[should_panic]
    fn test_panic_on_uninitialized() {
        let _ = Once::<String>::new().deref();
    }
}

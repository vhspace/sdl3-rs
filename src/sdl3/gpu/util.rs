use std::{mem::ManuallyDrop, ptr::NonNull};

use crate::{get_error, Error};

use super::Extern;


#[inline(always)]
pub(super) fn nonnull_ext_or_get_error<T>(ptr: *mut T) -> Result<NonNull<Extern<T>>, Error> {
    NonNull::new(ptr.cast()).ok_or_else(|| get_error())
}

/// Calls the closure when being dropped
pub(super) struct Defer<F: FnOnce()>(ManuallyDrop<F>);
impl<F: FnOnce()> Defer<F> {
    pub(super) fn new(f: F) -> Self {
        Self(ManuallyDrop::new(f))
    }
}

impl<F: FnOnce()> Drop for Defer<F> {
    fn drop(&mut self) {
        // Safety: This is the last (and only) use of the value
        unsafe {
            let f = ManuallyDrop::take(&mut self.0);
            f();
        }
    }
}
use core::marker::PhantomData;
use core::ptr::{NonNull, drop_in_place};
use core::mem;
use core::ops::{DerefMut, Deref};
use core::fmt;
use core::hash;
use core::borrow;

use super::Retained;
use crate::runtime::{self, Object};

/// A smart pointer that strongly references and owns an Objective-C object.
///
/// The fact that we own the pointer means that we're safe to mutate it, hence
/// why this implements [`DerefMut`].
///
/// This is guaranteed to have the same size as the underlying pointer.
///
/// TODO: Explain similarities to [`Box`].
///
/// TODO: Explain this vs. [`Retained`]
#[repr(transparent)]
pub struct Owned<T> {
    /// The pointer is always retained.
    pub(super) ptr: NonNull<T>, // Covariant
    phantom: PhantomData<T>, // Necessary for dropcheck
}

// SAFETY: TODO
unsafe impl<T: Send> Send for Owned<T> {}

// SAFETY: TODO
unsafe impl<T: Sync> Sync for Owned<T> {}

impl<T> Owned<T> {
    #[inline]
    pub unsafe fn new(ptr: NonNull<T>) -> Self {
        Self {
            ptr,
            phantom: PhantomData,
        }
    }

    // TODO: Unsure how the API should look...
    #[inline]
    pub unsafe fn retain(ptr: NonNull<T>) -> Self {
        Self::from_retained(Retained::retain(ptr))
    }

    /// TODO
    ///
    /// # Safety
    ///
    /// The given [`Retained`] must be the only reference to the object
    /// anywhere in the program - even in other Objective-C code.
    #[inline]
    pub unsafe fn from_retained(obj: Retained<T>) -> Self {
        let ptr = mem::ManuallyDrop::new(obj).ptr;
        Self {
            ptr,
            phantom: PhantomData,
        }
    }
}

// TODO: #[may_dangle]
// https://doc.rust-lang.org/nightly/nomicon/dropck.html
impl<T> Drop for Owned<T> {
    /// Releases the retained object
    ///
    /// This is guaranteed to be the last destructor that runs, in contrast to
    /// [`Retained`], which means that we can run the [`Drop`] implementation
    /// on the contained object as well.
    #[inline]
    fn drop(&mut self) {
        let ptr = self.ptr;
        unsafe {
            drop_in_place(ptr.as_ptr());
            // Construct a new `Retained`, which will be dropped immediately
            Retained::new(ptr);
        };
    }
}

// Note: `Clone` is not implemented for this!

impl<T> Deref for Owned<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: TODO
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> DerefMut for Owned<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: TODO
        unsafe { self.ptr.as_mut() }
    }
}

// TODO: impl PartialEq, PartialOrd, Ord and Eq

impl<T: fmt::Display> fmt::Display for Owned<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: fmt::Debug> fmt::Debug for Owned<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T> fmt::Pointer for Owned<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&self.ptr.as_ptr(), f)
    }
}

impl<T: hash::Hash> hash::Hash for Owned<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        (&**self).hash(state)
    }
}

// TODO: impl Fn traits? See `boxed_closure_impls`

// TODO: CoerceUnsized

impl<T> borrow::Borrow<T> for Owned<T> {
    fn borrow(&self) -> &T {
        &**self
    }
}

impl<T> borrow::BorrowMut<T> for Owned<T> {
    fn borrow_mut(&mut self) -> &mut T {
        &mut **self
    }
}

impl<T> AsRef<T> for Owned<T> {
    fn as_ref(&self) -> &T {
        &**self
    }
}

impl<T> AsMut<T> for Owned<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut **self
    }
}

// TODO: Comment on impl Unpin for Box
impl<T> Unpin for Owned<T> {}

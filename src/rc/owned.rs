use core::borrow;
use core::fmt;
use core::hash;
use core::marker::PhantomData;
use core::mem;
use core::ops::{Deref, DerefMut};
use core::ptr::{drop_in_place, NonNull};

use super::AutoreleasePool;
use super::Retained;

/// A smart pointer that strongly references and uniquely owns an Objective-C
/// object.
///
/// The fact that we uniquely own the pointer means that it's safe to mutate
/// it. As such, this implements [`DerefMut`].
///
/// This is guaranteed to have the same size as the underlying pointer.
///
/// # Cloning and [`Retained`]
///
/// This does not implement [`Clone`], but [`Retained`] has a [`From`]
/// implementation to convert from this, so you can easily reliquish ownership
/// and work with a clonable [`Retained`] pointer.
///
/// ```no_run
/// let obj: Owned<T> = ...;
/// let retained: Retained<T> = obj.into();
/// let cloned: Retained<T> = retained.clone();
/// ```
///
/// TODO: Explain similarities to [`Box`].
///
/// TODO: Explain this vs. [`Retained`]
#[repr(transparent)]
pub struct Owned<T> {
    /// The pointer is always retained.
    ptr: NonNull<T>, // We are the unique owner of T, so covariance is correct
    phantom: PhantomData<T>, // Necessary for dropck
}

/// `Owned` pointers are `Send` if `T` is `Send` because they give the same
/// access as having a T directly.
unsafe impl<T: Send> Send for Owned<T> {}

/// `Owned` pointers are `Sync` if `T` is `Sync` because they give the same
/// access as having a `T` directly.
unsafe impl<T: Sync> Sync for Owned<T> {}

// TODO: Unsure how the API should look...
impl<T> Owned<T> {
    /// Create a new `Owned` pointer to the object.
    ///
    /// Uses a retain count that has been handed off from somewhere else,
    /// usually Objective-C methods like `init`, `alloc`, `new`, or `copy`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there are no other pointers or references
    /// to the same object, and the given pointer is not be used afterwards.
    ///
    /// Additionally, the given object pointer must have +1 retain count.
    ///
    /// And lastly, the object pointer must be valid as a mutable reference
    /// (non-null, aligned, dereferencable, initialized and upholds aliasing
    /// rules, see the [`std::ptr`] module for more information).
    ///
    /// # Example
    ///
    /// ```rust
    /// let obj: &mut Object = unsafe { msg_send![cls, alloc] };
    /// let obj: Owned<Object> = unsafe { Owned::new(msg_send![obj, init]) };
    /// // Or in this case simply just:
    /// let obj: Owned<Object> = unsafe { Owned::new(msg_send![cls, new]) };
    /// ```
    #[inline]
    // Note: We don't take a mutable reference as a parameter since it would
    // be too easy to accidentally create two aliasing mutable references.
    pub unsafe fn new(ptr: *mut T) -> Self {
        Self {
            // SAFETY: Upheld by the caller
            ptr: NonNull::new_unchecked(ptr),
            phantom: PhantomData,
        }
    }

    /// Acquires a `*mut` pointer to the object.
    #[inline]
    pub fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Construct an `Owned` pointer from a `Retained` pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there are no other pointers to the same
    /// object (which also means that the given [`Retained`] should have a
    /// retain count of exactly 1 in almost all cases).
    #[inline]
    pub unsafe fn from_retained(obj: Retained<T>) -> Self {
        // SAFETY: The pointer is guaranteed by `Retained` to be NonNull
        let ptr = NonNull::new_unchecked(mem::ManuallyDrop::new(obj).as_ptr() as *mut T);
        Self {
            ptr,
            phantom: PhantomData,
        }
    }

    /// Autoreleases the retained pointer, meaning that the object is not
    /// immediately released, but will be when the innermost / current
    /// autorelease pool is drained.
    #[doc(alias = "objc_autorelease")]
    #[must_use = "If you don't intend to use the object any more, just drop it as usual"]
    #[inline]
    pub fn autorelease<'p>(self, pool: &'p AutoreleasePool) -> &'p mut T {
        let retained: Retained<T> = self.into();
        let ptr = retained.autorelease(pool) as *const T as *mut T;
        // SAFETY: The pointer was previously `Owned`, so is safe to be mutable
        unsafe { &mut *ptr }
    }
}

/// `#[may_dangle]` (see [this][dropck_eyepatch]) would not be safe here,
/// since we cannot verify that a `dealloc` method doesn't access borrowed
/// data.
///
/// [dropck_eyepatch]: https://doc.rust-lang.org/nightly/nomicon/dropck.html#an-escape-hatch
impl<T> Drop for Owned<T> {
    /// Releases the retained object.
    ///
    /// This is guaranteed to be the last destructor that runs, in contrast to
    /// [`Retained`], which means that we can run the [`Drop`] implementation
    /// on the contained object as well.
    #[inline]
    fn drop(&mut self) {
        let ptr = self.as_ptr();
        unsafe {
            drop_in_place(ptr);
            // Construct a new `Retained`, which will be dropped immediately
            Retained::new(ptr);
        };
    }
}

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
        fmt::Pointer::fmt(&self.as_ptr(), f)
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

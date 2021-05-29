use core::borrow;
use core::fmt;
use core::hash;
use core::marker::{PhantomData, Unpin};
use core::mem;
use core::ops::Deref;
use core::ptr::NonNull;

use crate::runtime::{self, Object};
use super::Owned;

/// A smart pointer that strongly references an object, ensuring it won't be
/// deallocated.
///
/// This is guaranteed to have the same size as the underlying pointer.
///
/// ## Caveats
///
/// If the inner type implements [`Drop`], that implementation will not be
/// called, since there is no way to ensure that the Objective-C runtime will
/// do so. If you need to run some code when the object is destroyed,
/// implement the `dealloc` selector instead.
///
/// TODO: Explain similarities with `Arc` and `RefCell`.
#[repr(transparent)]
pub struct Retained<T> {
    /// A pointer to the contained object.
    ///
    /// It is important that this is `NonNull`, since we want to dereference
    /// it later.
    ///
    /// Usually the contained object would be an [extern type][extern-type-rfc]
    /// (when that gets stabilized), or a type such as:
    /// ```
    /// pub struct MyType {
    ///     _data: [u8; 0], // TODO: `UnsafeCell`?
    /// }
    /// ```
    ///
    /// DSTs that carry metadata cannot be used here, so unsure if we should
    /// have a `?Sized` bound?
    ///
    /// TODO:
    /// https://doc.rust-lang.org/book/ch19-04-advanced-types.html#dynamically-sized-types-and-the-sized-trait
    /// https://doc.rust-lang.org/nomicon/exotic-sizes.html
    /// https://doc.rust-lang.org/core/ptr/trait.Pointee.html
    /// https://doc.rust-lang.org/core/ptr/traitalias.Thin.html
    ///
    /// [extern-type-rfc]: https://github.com/rust-lang/rfcs/blob/master/text/1861-extern-types.md
    ptr: NonNull<T>, // Covariant
    phantom: PhantomData<T>,
}

impl<T> Retained<T> {
    /// Constructs a `Retained<T>` to an object that already has a +1 retain
    /// count. This will not retain the object.
    ///
    /// When dropped, the object will be released.
    ///
    /// # Safety
    ///
    /// The caller must ensure the given object pointer is valid, and has +1
    /// retain count.
    ///
    /// TODO: Something about there not being any mutable references.
    #[inline]
    pub const unsafe fn new(ptr: NonNull<T>) -> Self {
        Retained {
            ptr,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub const fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Retains the given object pointer.
    ///
    /// When dropped, the object will be released.
    ///
    /// # Safety
    ///
    /// The caller must ensure the given object pointer is valid.
    #[doc(alias = "objc_retain")]
    #[inline]
    pub unsafe fn retain(ptr: NonNull<T>) -> Self {
        // SAFETY: The caller upholds that the pointer is valid
        let rtn = runtime::objc_retain(ptr.as_ptr() as *mut Object);
        debug_assert_eq!(rtn, ptr.as_ptr() as *mut Object);
        Retained {
            ptr,
            phantom: PhantomData,
        }
    }

    /// Autoreleases the retained pointer, meaning that the object is not
    /// immediately released, but will be when the innermost / current
    /// autorelease pool is drained.
    ///
    /// A pointer to the object is returned, but it's validity is only until
    /// guaranteed until the innermost pool is drained.
    #[doc(alias = "objc_autorelease")]
    #[must_use = "If you don't intend to use the object any more, just drop it as usual"]
    #[inline]
    pub fn autorelease(self) -> NonNull<T> {
        let ptr = mem::ManuallyDrop::new(self).ptr;
        // SAFETY: The `ptr` is guaranteed to be valid and have at least one
        // retain count.
        // And because of the ManuallyDrop, we don't call the Drop
        // implementation, so the object won't also be released there.
        unsafe { runtime::objc_autorelease(ptr.as_ptr() as *mut Object) };
        ptr
    }

    #[cfg(test)]
    #[doc(alias = "retainCount")]
    pub fn retain_count(&self) -> usize {
        unsafe { msg_send![self.ptr.as_ptr() as *mut Object, retainCount] }
    }
}

// TODO: #[may_dangle]
// https://doc.rust-lang.org/nightly/nomicon/dropck.html
impl<T> Drop for Retained<T> {
    /// Releases the retained object
    #[doc(alias = "objc_release")]
    #[doc(alias = "release")]
    #[inline]
    fn drop(&mut self) {
        // SAFETY: The `ptr` is guaranteed to be valid and have at least one
        // retain count
        unsafe { runtime::objc_release(self.ptr.as_ptr() as *mut Object) };
    }
}

impl<T> Clone for Retained<T> {
    /// Makes a clone of the `Retained` object.
    ///
    /// This increases the object's reference count.
    #[doc(alias = "objc_retain")]
    #[doc(alias = "retain")]
    #[inline]
    fn clone(&self) -> Self {
        // SAFETY: The `ptr` is guaranteed to be valid
        unsafe { Self::retain(self.ptr) }
    }
}

impl<T> Deref for Retained<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: TODO
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: PartialEq> PartialEq for Retained<T> {
    #[inline]
    fn eq(&self, other: &Retained<T>) -> bool {
        &**self == &**other
    }

    #[inline]
    fn ne(&self, other: &Retained<T>) -> bool {
        &**self != &**other
    }
}

// TODO: impl PartialOrd, Ord and Eq

impl<T: fmt::Display> fmt::Display for Retained<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: fmt::Debug> fmt::Debug for Retained<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T> fmt::Pointer for Retained<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&self.as_ptr(), f)
    }
}

impl<T: hash::Hash> hash::Hash for Retained<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        (&**self).hash(state)
    }
}

impl<T> borrow::Borrow<T> for Retained<T> {
    fn borrow(&self) -> &T {
        &**self
    }
}

impl<T> AsRef<T> for Retained<T> {
    fn as_ref(&self) -> &T {
        &**self
    }
}

impl<T> Unpin for Retained<T> {}

impl<T> From<Owned<T>> for Retained<T> {
    fn from(obj: Owned<T>) -> Self {
        // SAFETY: TODO
        unsafe { Self::new(obj.ptr) }
    }
}

#[cfg(test)]
mod tests {
    use core::mem::size_of;
    use core::ptr::NonNull;

    use super::Retained;
    use crate::runtime::Object;

    pub struct TestType {
        _data: [u8; 0], // TODO: `UnsafeCell`?
    }

    #[test]
    fn test_size_of() {
        assert_eq!(size_of::<Retained<TestType>>(), size_of::<&TestType>());
        assert_eq!(
            size_of::<Option<Retained<TestType>>>(),
            size_of::<&TestType>()
        );
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    #[test]
    fn test_clone() {
        // TODO: Maybe make a way to return `Retained` directly?
        let obj: *mut Object = unsafe { msg_send![class!(NSObject), new] };
        let obj: Retained<Object> = unsafe { Retained::new(NonNull::new(obj).unwrap()) };
        assert!(obj.retain_count() == 1);

        let cloned = obj.clone();
        assert!(cloned.retain_count() == 2);
        assert!(obj.retain_count() == 2);

        drop(obj);
        assert!(cloned.retain_count() == 1);
    }
}

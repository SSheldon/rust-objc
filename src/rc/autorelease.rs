use crate::runtime::{objc_autoreleasePoolPop, objc_autoreleasePoolPush};
use std::os::raw::c_void;

/// An Objective-C autorelease pool.
///
/// The pool is drained when dropped.
///
/// This is not `Send`, since `objc_autoreleasePoolPop` must be called on the
/// same thread.
///
/// And this is not `Sync`, since you can only autorelease a reference to a
/// pool on the current thread.
///
/// See [the clang documentation][clang-arc] and
/// [this apple article][memory-mgmt] for more information on automatic
/// reference counting.
///
/// [clang-arc]: https://clang.llvm.org/docs/AutomaticReferenceCounting.html
/// [memory-mgmt]: https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/MemoryMgmt/Articles/MemoryMgmt.html
pub struct AutoreleasePool {
    context: *mut c_void,
}

/// ```rust,compile_fail
/// use objc::rc::AutoreleasePool;
/// fn needs_sync<T: Send>() {}
/// needs_sync::<AutoreleasePool>();
/// ```
/// ```rust,compile_fail
/// use objc::rc::AutoreleasePool;
/// fn needs_send<T: Send>() {}
/// needs_send::<AutoreleasePool>();
/// ```
#[cfg(doctest)]
pub struct AutoreleasePoolNotSendNorSync;

impl AutoreleasePool {
    /// Construct a new autorelease pool.
    ///
    /// Use the [`autoreleasepool`] block for a safe alternative.
    ///
    /// # Safety
    ///
    /// The caller must ensure that when handing out `&'p AutoreleasePool` to
    /// functions that this is the innermost pool.
    ///
    /// Additionally, the pools must be dropped in the same order they were
    /// created.
    #[doc(alias = "objc_autoreleasePoolPush")]
    unsafe fn new() -> Self {
        // TODO: Make this function pub when we're more certain of the API
        Self {
            context: objc_autoreleasePoolPush(),
        }
    }

    // TODO: Add helper functions to ensure (with debug_assertions) that the
    // pool is innermost when its lifetime is tied to a reference.
}

impl Drop for AutoreleasePool {
    /// Drains the autoreleasepool.
    ///
    /// The [clang documentation] says that `@autoreleasepool` blocks are not
    /// drained when exceptions occur because:
    ///
    /// > Not draining the pool during an unwind is apparently required by the
    /// > Objective-C exceptions implementation.
    ///
    /// This was true in the past, but since [revision `371`] of
    /// `objc-exception.m` (ships with MacOS 10.5) the exception is now
    /// retained when `@throw` is encountered.
    ///
    /// Hence it is safe to drain the pool when unwinding.
    ///
    /// [clang documentation]: https://clang.llvm.org/docs/AutomaticReferenceCounting.html#autoreleasepool
    /// [revision `371`]: https://opensource.apple.com/source/objc4/objc4-371/runtime/objc-exception.m.auto.html
    #[doc(alias = "objc_autoreleasePoolPop")]
    fn drop(&mut self) {
        unsafe { objc_autoreleasePoolPop(self.context) }
    }
}

// TODO:
// #![feature(negative_impls)]
// #![feature(auto_traits)]
// /// A trait for the sole purpose of ensuring we can't pass an `&AutoreleasePool`
// /// through to the closure inside `autoreleasepool`
// pub unsafe auto trait AutoreleaseSafe {}
// // TODO: Unsure how negative impls work exactly
// unsafe impl !AutoreleaseSafe for AutoreleasePool {}
// unsafe impl !AutoreleaseSafe for &AutoreleasePool {}
// unsafe impl !AutoreleaseSafe for &mut AutoreleasePool {}

/// Execute `f` in the context of a new autorelease pool. The pool is drained
/// after the execution of `f` completes.
///
/// This corresponds to `@autoreleasepool` blocks in Objective-C and Swift.
///
/// The pool is passed as a reference to the enclosing function to give it a
/// lifetime parameter that autoreleased objects can refer to.
///
/// # Examples
///
/// ```rust
/// use objc::{class, msg_send};
/// use objc::rc::{autoreleasepool, AutoreleasePool};
/// use objc::runtime::Object;
///
/// fn needs_lifetime_from_pool<'p>(_pool: &'p AutoreleasePool) -> &'p mut Object {
///     let obj: *mut Object = unsafe { msg_send![class!(NSObject), new] };
///     let obj: *mut Object = unsafe { msg_send![obj, autorelease] };
///     // SAFETY: Lifetime bounded by the pool
///     unsafe { &mut *obj }
/// }
///
/// autoreleasepool(|pool| {
///     let obj = needs_lifetime_from_pool(pool);
///     // Use `obj`
/// });
///
/// // `obj` is deallocated when the pool ends
/// ```
///
/// ```rust,compile_fail
/// # use objc::{class, msg_send};
/// # use objc::rc::{autoreleasepool, AutoreleasePool};
/// # use objc::runtime::Object;
/// #
/// # fn needs_lifetime_from_pool<'p>(_pool: &'p AutoreleasePool) -> &'p mut Object {
/// #     let obj: *mut Object = unsafe { msg_send![class!(NSObject), new] };
/// #     let obj: *mut Object = unsafe { msg_send![obj, autorelease] };
/// #     unsafe { &mut *obj }
/// # }
/// #
/// // Fails to compile because `obj` does not live long enough for us to
/// // safely take it out of the pool.
///
/// let obj = autoreleasepool(|pool| {
///     let obj = needs_lifetime_from_pool(pool);
///     // Use `obj`
///     obj
/// });
/// ```
///
/// TODO: More examples.
pub fn autoreleasepool<T, F>(f: F) -> T
where
    for<'p> F: FnOnce(&'p AutoreleasePool) -> T, // + AutoreleaseSafe,
{
    let pool = unsafe { AutoreleasePool::new() };
    f(&pool)
}

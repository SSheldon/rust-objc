/*!
Utilities for reference-counting Objective-C objects.

These utilities provide ARC-like semantics in Rust. They are not intended to
provide a fully safe interface, but can be useful when writing higher-level
Rust wrappers for Objective-C code.

For more information on Objective-C's reference counting, see Apple's documentation:
<https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/MemoryMgmt/Articles/MemoryMgmt.html>
*/

mod strong;
mod weak;

pub use self::strong::StrongPtr;
pub use self::weak::WeakPtr;

// These tests use NSObject, which isn't present for GNUstep
#[cfg(all(test, any(target_os = "macos", target_os = "ios")))]
mod tests {
    use runtime::Object;
    use super::StrongPtr;

    #[test]
    fn test_strong_clone() {
        fn retain_count(obj: *mut Object) -> usize {
            unsafe { msg_send![obj, retainCount] }
        }

        let obj = unsafe {
            StrongPtr::new(msg_send![class!(NSObject), new])
        };
        assert!(retain_count(*obj) == 1);

        let cloned = obj.clone();
        assert!(retain_count(*cloned) == 2);
        assert!(retain_count(*obj) == 2);

        drop(obj);
        assert!(retain_count(*cloned) == 1);
    }

    #[test]
    fn test_weak() {
        let obj = unsafe {
            StrongPtr::new(msg_send![class!(NSObject), new])
        };
        let weak = obj.weak();

        let strong = weak.load();
        assert!(*strong == *obj);
        drop(strong);

        drop(obj);
        assert!(weak.load().is_null());
    }

    #[test]
    fn test_weak_copy() {
        let obj = unsafe {
            StrongPtr::new(msg_send![class!(NSObject), new])
        };
        let weak = obj.weak();

        let weak2 = weak.clone();
        let strong = weak2.load();
        assert!(*strong == *obj);
    }
}

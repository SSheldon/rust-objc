// These tests use NSObject, which isn't present for GNUstep
#![cfg(any(target_os = "macos", target_os = "ios"))]

use objc::rc::{StrongPtr, autoreleasepool};
use objc::runtime::Object;
use objc::{class, msg_send};

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

#[test]
fn test_autorelease() {
    let obj = unsafe {
        StrongPtr::new(msg_send![class!(NSObject), new])
    };

    fn retain_count(obj: *mut Object) -> usize {
        unsafe { msg_send![obj, retainCount] }
    }
    let cloned = obj.clone();

    autoreleasepool(|| {
                    obj.autorelease();
                    assert!(retain_count(*cloned) == 2);
    });

    // make sure that the autoreleased value has been released
    assert!(retain_count(*cloned) == 1);
}

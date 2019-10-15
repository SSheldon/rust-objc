#![cfg(any(target_os = "macos", target_os = "ios"))]

extern crate objc;

use objc::{class, msg_send, sel};
use objc::runtime::Object;

#[test]
fn use_class_and_msg_send() {
    unsafe {
        let cls = class!(NSObject);
        let obj = msg_send![cls, new => *mut Object];
        let _hash = msg_send![obj, hash => usize];
        msg_send![obj, release => ()];
    }
}

#[test]
fn use_sel() {
    let _sel = sel!(description);
    let _sel = sel!(setObject:forKey:);
}


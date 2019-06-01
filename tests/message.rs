use objc::runtime::Object;
use objc::{Message, msg_send, sel};

#[path = "common/mod.rs"]
mod test_utils;

#[test]
fn test_send_message() {
    let obj = test_utils::custom_object();
    let result: u32 = unsafe {
        let _: () = msg_send![obj, setFoo:4u32];
        msg_send![obj, foo]
    };
    assert!(result == 4);
}

#[test]
fn test_send_message_stret() {
    let obj = test_utils::custom_object();
    let result: test_utils::CustomStruct = unsafe {
        msg_send![obj, customStruct]
    };
    let expected = test_utils::CustomStruct { a: 1, b:2, c: 3, d: 4 };
    assert!(result == expected);
}

#[cfg(not(feature = "verify_message"))]
#[test]
fn test_send_message_nil() {
    let nil: *mut Object = ::std::ptr::null_mut();
    let result: usize = unsafe {
        msg_send![nil, hash]
    };
    assert!(result == 0);

    let result: *mut Object = unsafe {
        msg_send![nil, description]
    };
    assert!(result.is_null());

    let result: f64 = unsafe {
        msg_send![nil, doubleValue]
    };
    assert!(result == 0.0);
}

#[test]
fn test_send_message_super() {
    let obj = test_utils::custom_subclass_object();
    let superclass = test_utils::custom_class();
    unsafe {
        let _: () = msg_send![obj, setFoo:4u32];
        let foo: u32 = msg_send![super(obj, superclass), foo];
        assert!(foo == 4);

        // The subclass is overriden to return foo + 2
        let foo: u32 = msg_send![obj, foo];
        assert!(foo == 6);
    }
}

#[test]
fn test_verify_message() {
    let obj = test_utils::custom_object();
    assert!(obj.verify_message::<(), u32>(sel!(foo)).is_ok());
    assert!(obj.verify_message::<(u32,), ()>(sel!(setFoo:)).is_ok());

    // Incorrect types
    assert!(obj.verify_message::<(), u64>(sel!(setFoo:)).is_err());
    // Unimplemented selector
    assert!(obj.verify_message::<(u32,), ()>(sel!(setFoo)).is_err());
}
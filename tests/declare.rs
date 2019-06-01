use objc::msg_send;

#[path = "common/mod.rs"]
mod test_utils;

#[test]
fn test_custom_class() {
    // Registering the custom class is in test_utils
    let obj = test_utils::custom_object();
    unsafe {
        let _: () = msg_send![obj, setFoo:13u32];
        let result: u32 = msg_send![obj, foo];
        assert!(result == 13);
    }
}

#[test]
fn test_class_method() {
    let cls = test_utils::custom_class();
    unsafe {
        let result: u32 = msg_send![cls, classFoo];
        assert!(result == 7);
    }
}
use std::sync::{Once, ONCE_INIT};
use objc_test_utils;

use block::Block;
use runtime::{Class, Object, Sel};
use {ClassDecl, Encode, Id, MethodDecl};

pub fn sample_object() -> Id<Object> {
    let cls = Class::get("NSObject").unwrap();
    unsafe {
        let obj: *mut Object = msg_send![cls, alloc];
        let obj: *mut Object = msg_send![obj, init];
        Id::from_retained_ptr(obj)
    }
}

pub fn get_int_block_with(i: i32) -> Id<Block<(), i32>> {
    unsafe {
        let ptr = objc_test_utils::get_int_block_with(i);
        Id::from_retained_ptr(ptr as *mut _)
    }
}

pub fn get_add_block_with(i: i32) -> Id<Block<(i32,), i32>> {
    unsafe {
        let ptr = objc_test_utils::get_add_block_with(i);
        Id::from_retained_ptr(ptr as *mut _)
    }
}

pub fn invoke_int_block(block: &mut Block<(), i32>) -> i32 {
    let ptr = block as *mut _;
    unsafe {
        objc_test_utils::invoke_int_block(ptr as *mut _)
    }
}

pub fn invoke_add_block(block: &mut Block<(i32,), i32>, a: i32) -> i32 {
    let ptr = block as *mut _;
    unsafe {
        objc_test_utils::invoke_add_block(ptr as *mut _, a)
    }
}

#[derive(Eq, PartialEq)]
pub struct CustomStruct {
    pub a: u64,
    pub b: u64,
    pub c: u64,
    pub d: u64,
}

impl Encode for CustomStruct {
    fn code() -> &'static str { "{CustomStruct=QQQQ}" }
}

static REGISTER_CUSTOM_CLASS: Once = ONCE_INIT;

pub fn custom_class() -> &'static Class {
    REGISTER_CUSTOM_CLASS.call_once(|| {
        let superclass = Class::get("NSObject").unwrap();
        let mut decl = ClassDecl::new(superclass, "CustomObject").unwrap();

        extern fn custom_object_get_struct(_this: &Object, _cmd: Sel) -> CustomStruct {
            CustomStruct { a: 1, b: 2, c: 3, d: 4 }
        }
        let method = MethodDecl::new(sel!(customStruct),
            custom_object_get_struct as extern fn(&Object, Sel) -> CustomStruct);
        assert!(decl.add_method(method.unwrap()));

        decl.register();
    });

    Class::get("CustomObject").unwrap()
}

pub fn custom_object() -> Id<Object> {
    let cls = custom_class();
    unsafe {
        let obj: *mut Object = msg_send![cls, alloc];
        let obj: *mut Object = msg_send![obj, init];
        Id::from_retained_ptr(obj)
    }
}

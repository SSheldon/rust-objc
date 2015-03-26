use std::sync::{Once, ONCE_INIT};
use objc_test_utils;

use block::Block;
use declare::ClassDecl;
use encode;
use runtime::{Class, Object, Sel};
use {Encode, Encoding, Id};

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

unsafe impl Encode for CustomStruct {
    fn encode() -> Encoding {
        let code = encode!(struct CustomStruct { u64, u64, u64, u64 });
        encode::from_static_str(code)
    }
}

static REGISTER_CUSTOM_CLASS: Once = ONCE_INIT;

pub fn custom_class() -> &'static Class {
    REGISTER_CUSTOM_CLASS.call_once(|| {
        let superclass = Class::get("NSObject").unwrap();
        let mut decl = ClassDecl::new(superclass, "CustomObject").unwrap();

        decl.add_ivar::<u32>("_foo");

        extern fn custom_obj_set_foo(this: &mut Object, _cmd: Sel, foo: u32) {
            unsafe { this.set_ivar::<u32>("_foo", foo); }
        }

        extern fn custom_obj_get_foo(this: &Object, _cmd: Sel) -> u32 {
            unsafe { *this.get_ivar::<u32>("_foo") }
        }

        extern fn custom_obj_get_struct(_this: &Object, _cmd: Sel) -> CustomStruct {
            CustomStruct { a: 1, b: 2, c: 3, d: 4 }
        }

        extern fn custom_obj_class_method(_this: &Class, _cmd: Sel) -> u32 {
            7
        }

        unsafe {
            let set_foo: extern fn(&mut Object, Sel, u32) = custom_obj_set_foo;
            decl.add_method(sel!(setFoo:), set_foo);
            let get_foo: extern fn(&Object, Sel) -> u32 = custom_obj_get_foo;
            decl.add_method(sel!(foo), get_foo);
            let get_struct: extern fn(&Object, Sel) -> CustomStruct = custom_obj_get_struct;
            decl.add_method(sel!(customStruct), get_struct);
            let class_method: extern fn(&Class, Sel) -> u32 = custom_obj_class_method;
            decl.add_class_method(sel!(classFoo), class_method);
        }

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

static REGISTER_CUSTOM_SUBCLASS: Once = ONCE_INIT;

pub fn custom_subclass() -> &'static Class {
    REGISTER_CUSTOM_SUBCLASS.call_once(|| {
        let superclass = custom_class();
        let mut decl = ClassDecl::new(superclass, "CustomSubclassObject").unwrap();

        extern fn custom_subclass_get_foo(this: &Object, _cmd: Sel) -> u32 {
            let foo = unsafe {
                msg_send![super(this, custom_class()), foo]
            };
            foo + 2
        }

        unsafe {
            let get_foo: extern fn(&Object, Sel) -> u32 = custom_subclass_get_foo;
            decl.add_method(sel!(foo), get_foo);
        }

        decl.register();
    });

    Class::get("CustomSubclassObject").unwrap()
}

pub fn custom_subclass_object() -> Id<Object> {
    let cls = custom_subclass();
    unsafe {
        let obj: *mut Object = msg_send![cls, alloc];
        let obj: *mut Object = msg_send![obj, init];
        Id::from_retained_ptr(obj)
    }
}

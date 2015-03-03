use std::sync::{Once, ONCE_INIT};
use objc_test_utils;

use block::Block;
use declare::ClassDecl;
use runtime::{Class, Object, Sel};
use {Encode, Id, send_super_message};

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

        decl.add_ivar::<u32>("_foo");

        extern fn custom_obj_set_foo(this: &mut Object, _cmd: Sel, foo: u32) {
            unsafe { this.set_ivar::<u32>("_foo", foo); }
        }
        decl.add_method(sel!(setFoo:),
            custom_obj_set_foo as extern fn(&mut Object, Sel, u32));

        extern fn custom_obj_get_foo(this: &Object, _cmd: Sel) -> u32 {
            unsafe { *this.get_ivar::<u32>("_foo") }
        }
        decl.add_method(sel!(foo),
            custom_obj_get_foo as extern fn(&Object, Sel) -> u32);

        extern fn custom_obj_get_struct(_this: &Object, _cmd: Sel) -> CustomStruct {
            CustomStruct { a: 1, b: 2, c: 3, d: 4 }
        }
        decl.add_method(sel!(customStruct),
            custom_obj_get_struct as extern fn(&Object, Sel) -> CustomStruct);

        extern fn custom_obj_class_method(_this: &Class, _cmd: Sel) -> u32 {
            7
        }
        decl.add_class_method(sel!(classFoo),
            custom_obj_class_method as extern fn(&Class, Sel) -> u32);

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

        extern fn custom_subclass_get_foo(this: &Object, cmd: Sel) -> u32 {
            let superclass = custom_class();
            let foo = unsafe {
                send_super_message(&this, superclass, cmd, ())
            };
            foo + 2
        }
        decl.add_method(sel!(foo),
            custom_subclass_get_foo as extern fn(&Object, Sel) -> u32);

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

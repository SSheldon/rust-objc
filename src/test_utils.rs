use std::sync::{Once, ONCE_INIT};

use declare::ClassDecl;
use id::StrongPtr;
use runtime::{Class, Object, Sel};
use {Encode, Encoding};




#[cfg(feature="gnustep")]
#[link(name = "NSObject", kind = "static")]
extern {
}


pub fn sample_object() -> StrongPtr {
    let cls = Class::get("NSObject").unwrap();
    unsafe {
        let obj: *mut Object = msg_send![cls, alloc];
        let obj: *mut Object = msg_send![obj, init];
        StrongPtr::new(obj)
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
        let mut code = "{CustomStruct=".to_owned();
        for _ in 0..4 {
            code.push_str(u64::encode().as_str());
        }
        code.push_str("}");
        unsafe {
            Encoding::from_str(&code)
        }
    }
}

pub fn custom_class() -> &'static Class {
    static REGISTER_CUSTOM_CLASS: Once = ONCE_INIT;

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

pub fn custom_object() -> StrongPtr {
    let cls = custom_class();
    unsafe {
        let obj: *mut Object = msg_send![cls, alloc];
        let obj: *mut Object = msg_send![obj, init];
        StrongPtr::new(obj)
    }
}

pub fn custom_subclass() -> &'static Class {
    static REGISTER_CUSTOM_SUBCLASS: Once = ONCE_INIT;

    REGISTER_CUSTOM_SUBCLASS.call_once(|| {
        let superclass = custom_class();
        let mut decl = ClassDecl::new(superclass, "CustomSubclassObject").unwrap();

        extern fn custom_subclass_get_foo(this: &Object, _cmd: Sel) -> u32 {
            let foo: u32 = unsafe {
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

pub fn custom_subclass_object() -> StrongPtr {
    let cls = custom_subclass();
    unsafe {
        let obj: *mut Object = msg_send![cls, alloc];
        let obj: *mut Object = msg_send![obj, init];
        StrongPtr::new(obj)
    }
}

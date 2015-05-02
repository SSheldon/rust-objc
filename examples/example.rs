#[macro_use]
extern crate objc;

use objc::Encode;
use objc::runtime::{Class, Object};

/// Wrapper around an `Object` pointer that will release it when dropped.
struct StrongPtr(*mut Object);

impl std::ops::Deref for StrongPtr {
    type Target = Object;

    fn deref(&self) -> &Object {
        unsafe { &*self.0 }
    }
}

impl Drop for StrongPtr {
    fn drop(&mut self) {
        let _: () = unsafe { msg_send![self.0, release] };
    }
}

fn main() {
    // Get a class
    let cls = Class::get("NSObject").unwrap();
    println!("NSObject size: {}", cls.instance_size());

    // Inspect its ivars
    println!("NSObject ivars:");
    for ivar in cls.instance_variables().iter() {
        println!("{}", ivar.name());
    }

    // Allocate an instance
    let obj = unsafe {
        let obj: *mut Object = msg_send![cls, alloc];
        let obj: *mut Object = msg_send![obj, init];
        StrongPtr(obj)
    };
    println!("NSObject address: {:p}", &*obj);

    // Access an ivar of the object
    let isa: *const Class = unsafe {
        *obj.get_ivar("isa")
    };
    println!("NSObject isa: {:?}", isa);

    // Inspect a method of the class
    let hash_sel = sel!(hash);
    let hash_method = cls.instance_method(hash_sel).unwrap();
    let hash_return = hash_method.return_type();
    println!("-[NSObject hash] return type: {:?}", hash_return);
    assert!(hash_return == usize::encode());

    // Invoke a method on the object
    let hash: usize = unsafe {
        msg_send![obj, hash]
    };
    println!("NSObject hash: {}", hash);
}

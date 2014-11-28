#![feature(phase)]

#[phase(plugin, link)]
extern crate objc;

use objc::{encode, Id, ToMessage, WeakId};
use objc::runtime::{Class, Object, Sel};

fn main() {
    // Get a class
    let cls = Class::get("NSObject").unwrap();
    println!("NSObject size: {}", cls.instance_size());

    // Inspect its ivars
    println!("NSObject ivars:")
    for ivar in cls.instance_variables().as_slice().iter() {
        println!("{}", ivar.name());
    }

    // Allocate an instance
    let obj: Id<Object> = unsafe {
        let obj = msg_send![cls alloc];
        let obj = msg_send![obj init];
        Id::from_retained_ptr(obj)
    };
    println!("NSObject address: {}", obj.as_ptr());

    // Access an ivar of the object
    let isa: *const Class = unsafe {
        *obj.get_ivar("isa")
    };
    println!("NSObject isa: {}", isa);

    // Inspect a method of the class
    let hash_sel = Sel::register("hash");
    let hash_method = cls.instance_method(hash_sel).unwrap();
    let hash_return = hash_method.return_type();
    println!("-[NSObject hash] return type: {}", hash_return.as_str().unwrap());
    assert!(encode::<uint>() == hash_return.as_str().unwrap());

    // Invoke a method on the object
    let hash = unsafe {
        (msg_send![obj hash]) as uint
    };
    println!("NSObject hash: {}", hash);

    // Take a weak reference to the object
    let obj = obj.share();
    let weak = WeakId::new(&obj);
    println!("Weak reference is nil? {}", weak.load().is_none());

    println!("Releasing object");
    drop(obj);
    println!("Weak reference is nil? {}", weak.load().is_none());
}

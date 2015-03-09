#[macro_use]
extern crate objc;

use objc::{Encode, Id, WeakId};
use objc::runtime::{Class, Object};

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
    let obj: Id<Object> = unsafe {
        let obj: *mut Object = msg_send![cls, alloc];
        let obj: *mut Object = msg_send![obj, init];
        Id::from_retained_ptr(obj)
    };
    println!("NSObject address: {:?}", &*obj as *const Object);

    // Access an ivar of the object
    let isa: *const Class = unsafe {
        *obj.get_ivar("isa")
    };
    println!("NSObject isa: {:?}", isa);

    // Inspect a method of the class
    let hash_sel = sel!(hash);
    let hash_method = cls.instance_method(hash_sel).unwrap();
    let hash_return = hash_method.return_type();
    println!("-[NSObject hash] return type: {}", &*hash_return);
    assert!(usize::encode() == &*hash_return);

    // Invoke a method on the object
    let hash: usize = unsafe {
        msg_send![obj, hash]
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

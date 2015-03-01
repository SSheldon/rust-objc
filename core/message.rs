use std::marker::MarkerTrait;
use std::mem;

use runtime::{Class, Method, Object, Sel, Super, self};
use {encode, Encode};

/*
 The Sized bound on Message is unfortunate; ideally, objc objects would not be
 treated as Sized. However, rust won't allow casting a dynamically-sized type
 pointer to an Object pointer, because dynamically-sized types can have fat
 pointers (two words) instead of real pointers.
 */
/// Types that may be sent Objective-C messages.
/// For example: objects, classes, and blocks.
pub unsafe trait Message : MarkerTrait { }

unsafe impl Message for Object { }

unsafe impl Message for Class { }

/// A trait for converting to an `id` pointer that may be sent Objective-C
/// messages.
pub trait ToMessage {
    /// Returns self as an `id` pointer, a pointer to an `Object`.
    fn as_id_ptr(&self) -> *mut Object;

    /// Returns true if self is nil.
    fn is_nil(&self) -> bool {
        self.as_id_ptr().is_null()
    }
}

impl<T> ToMessage for *const T where T: Message {
    fn as_id_ptr(&self) -> *mut Object {
        *self as *mut Object
    }
}

impl<T> ToMessage for *mut T where T: Message {
    fn as_id_ptr(&self) -> *mut Object {
        *self as *mut Object
    }
}

impl<'a, T> ToMessage for &'a T where T: Message {
    fn as_id_ptr(&self) -> *mut Object {
        *self as *const T as *mut Object
    }
}

impl<'a, T> ToMessage for &'a mut T where T: Message {
    fn as_id_ptr(&self) -> *mut Object {
        *self as *mut T as *mut Object
    }
}

#[cfg(target_arch = "x86_64")]
fn msg_send_fn<R>() -> unsafe extern fn(*mut Object, Sel, ...) -> R {
    if mem::size_of::<R>() <= 16 {
        unsafe { mem::transmute(runtime::objc_msgSend) }
    } else {
        unsafe { mem::transmute(runtime::objc_msgSend_stret) }
    }
}

#[cfg(target_arch = "x86_64")]
fn msg_send_super_fn<R>() -> unsafe extern fn(*const Super, Sel, ...) -> R {
    if mem::size_of::<R>() <= 16 {
        unsafe { mem::transmute(runtime::objc_msgSendSuper) }
    } else {
        unsafe { mem::transmute(runtime::objc_msgSendSuper_stret) }
    }
}

/// Types that may be used as the arguments of an Objective-C message.
pub trait MessageArguments {
    /// Sends a message to the given obj with the given selector and self as
    /// the arguments.
    ///
    /// It is recommended to use the `msg_send!` macro rather than calling this
    /// method directly.
    unsafe fn send<T, R>(self, obj: &T, sel: Sel) -> R where T: ToMessage;

    /// Sends a message to the superclass of an instance of a class with self
    /// as the arguments.
    unsafe fn send_super<T, R>(self, obj: &T, superclass: &Class, sel: Sel) -> R
            where T: ToMessage;
}

macro_rules! message_args_impl {
    ($($a:ident : $t:ident),*) => (
        impl<$($t),*> MessageArguments for ($($t,)*) {
            unsafe fn send<T, R>(self, obj: &T, sel: Sel) -> R where T: ToMessage {
                let msg_send_fn = msg_send_fn::<R>();
                let msg_send_fn: unsafe extern fn(*mut Object, Sel $(, $t)*) -> R =
                    mem::transmute(msg_send_fn);
                let obj_ptr = obj.as_id_ptr();
                let ($($a,)*) = self;
                msg_send_fn(obj_ptr, sel $(, $a)*)
            }

            unsafe fn send_super<T, R>(self, obj: &T, superclass: &Class, sel: Sel) -> R
                    where T: ToMessage {
                let msg_send_fn = msg_send_super_fn::<R>();
                let msg_send_fn: unsafe extern fn(*const Super, Sel $(, $t)*) -> R =
                    mem::transmute(msg_send_fn);
                let sup = Super { receiver: obj.as_id_ptr(), superclass: superclass };
                let ($($a,)*) = self;
                msg_send_fn(&sup, sel $(, $a)*)
            }
        }
    );
}

message_args_impl!();
message_args_impl!(a: A);
message_args_impl!(a: A, b: B);
message_args_impl!(a: A, b: B, c: C);
message_args_impl!(a: A, b: B, c: C, d: D);
message_args_impl!(a: A, b: B, c: C, d: D, e: E);
message_args_impl!(a: A, b: B, c: C, d: D, e: E, f: F);
message_args_impl!(a: A, b: B, c: C, d: D, e: E, f: F, g: G);
message_args_impl!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H);
message_args_impl!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I);
message_args_impl!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J);
message_args_impl!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J, k: K);
message_args_impl!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J, k: K, l: L);

#[allow(dead_code)]
fn verify_message_arguments(types: &[&str], method: &Method) -> Result<(), String> {
    let count = 2 + types.len();
    let expected_count = method.arguments_count();
    if count != expected_count {
        return Err(format!("Method {:?} accepts {} arguments, but {} were given",
            method.name(), expected_count, count));
    }

    let expected_types = (2..expected_count).map(|i| method.argument_type(i));
    for (&arg, expected) in types.iter().zip(expected_types) {
        let expected = match expected {
            Some(s) => s,
            None => return Err(format!("Method {:?} doesn't expect argument with type code {}",
                method.name(), arg)),
        };
        if arg != &*expected {
            return Err(format!("Method {:?} expected argument with type code {} but was given {}",
                method.name(), &*expected, arg));
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn verify_message_signature<A, R>(cls: &Class, sel: Sel) -> Result<(), String>
        where A: MessageArguments, R: Encode {
    let method = match cls.instance_method(sel) {
        Some(method) => method,
        None => return Err(format!("Method {:?} not found on class {:?}",
            sel, cls)),
    };

    let ret = encode::<R>();
    let expected_ret = method.return_type();
    // Allow encoding "oneway void" (Vv) as "void" (v)
    if &*expected_ret != ret && !(&*expected_ret == "Vv" && ret == "v") {
        return Err(format!("Return type code {} does not match expected {} for method {:?} on class {:?}",
            ret, &*expected_ret, sel, cls));
    }

    // I don't think either of these can happen, but just to be safe...
    let accepts_self_arg = match method.argument_type(0) {
        Some(s) => &*s == "@",
        None => false,
    };
    if !accepts_self_arg {
        return Err(format!("Method {:?} of class {:?} doesn't accept an argument for self",
            sel, cls));
    }
    let accepts_cmd_arg = match method.argument_type(1) {
        Some(s) => &*s == ":",
        None => false,
    };
    if !accepts_cmd_arg {
        return Err(format!("Method {:?} of class {:?} doesn't accept an argument for the selector",
            sel, cls));
    }

    Ok(())
}

#[cfg(any(ndebug, not(feature = "verify_message_encode")))]
pub unsafe fn send_message<T, A, R>(obj: &T, sel: Sel, args: A) -> R
        where T: ToMessage, A: MessageArguments {
    args.send(obj, sel)
}

/**
Sends a message to an object with selector `sel` and arguments `args`.
This function will choose the correct version of `objc_msgSend` based on the
return type. For more information, see Apple's documenation:
https://developer.apple.com/library/mac/documentation/Cocoa/Reference/ObjCRuntimeRef/index.html#//apple_ref/doc/uid/TP40001418-CH1g-88778

When the verify_message_encode feature is defined, at runtime in debug
builds this function will verify that the encoding of the return type
matches the encoding of the method.

# Example
``` no_run
# #[macro_use] extern crate objc;
# use objc::send_message;
# use objc::runtime::Object;
# fn main() {
# unsafe {
let dict: *mut Object;
let key: *const Object;
# let dict: *mut Object = 0 as *mut Object;
# let key: *const Object = 0 as *const Object;
let obj: *const Object = send_message(&dict, sel!(objectForKey:), (key,));
let _: () = send_message(&dict, sel!(setObject:forKey:), (obj, key));
# }
# }
```
*/
#[cfg(all(not(ndebug), feature = "verify_message_encode"))]
pub unsafe fn send_message<T, A, R>(obj: &T, sel: Sel, args: A) -> R
        where T: ToMessage, A: MessageArguments, R: Encode {
    let cls = match obj.as_id_ptr().as_ref() {
        Some(obj) => obj.class(),
        None => panic!("Messaging {:?} to nil", sel),
    };
    match verify_message_signature::<A, R>(cls, sel) {
        Err(s) => panic!("Verify message failed: {}", s),
        Ok(_) => args.send(obj, sel),
    }
}

#[cfg(any(ndebug, not(feature = "verify_message_encode")))]
pub unsafe fn send_super_message<T, A, R>(
        obj: &T, superclass: &Class, sel: Sel, args: A) -> R
        where T: ToMessage, A: MessageArguments {
    args.send_super(obj, superclass, sel)
}

/// Sends a message to the superclass of an instance of a class.
#[cfg(all(not(ndebug), feature = "verify_message_encode"))]
pub unsafe fn send_super_message<T, A, R>(
        obj: &T, superclass: &Class, sel: Sel, args: A) -> R
        where T: ToMessage, A: MessageArguments, R: Encode {
    if obj.is_nil() {
        panic!("Messaging {:?} to nil", sel);
    }
    match verify_message_signature::<A, R>(superclass, sel) {
        Err(s) => panic!("Verify message failed: {}", s),
        Ok(_) => args.send_super(obj, superclass, sel),
    }
}

#[cfg(test)]
mod tests {
    use runtime::Object;
    use test_utils;
    use super::send_message;

    #[test]
    fn test_send_message() {
        let obj = test_utils::sample_object();
        let result: *const Object = unsafe {
            send_message(&obj, sel!(self), ())
        };
        assert!(&*obj as *const Object == result);
    }

    #[test]
    fn test_send_message_stret() {
        let obj = test_utils::custom_object();
        let result: test_utils::CustomStruct = unsafe {
            send_message(&obj, sel!(customStruct), ())
        };
        let expected = test_utils::CustomStruct { a: 1, b:2, c: 3, d: 4 };
        assert!(result == expected);
    }
}

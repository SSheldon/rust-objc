use std::marker::PhantomFn;
use std::mem;

use runtime::{Class, Object, Sel, Super, self};

/*
 The Sized bound on Message is unfortunate; ideally, objc objects would not be
 treated as Sized. However, rust won't allow casting a dynamically-sized type
 pointer to an Object pointer, because dynamically-sized types can have fat
 pointers (two words) instead of real pointers.
 */
/// Types that may be sent Objective-C messages.
/// For example: objects, classes, and blocks.
pub unsafe trait Message : PhantomFn<Self> { }

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

/**
Sends a message to an object with selector `sel` and arguments `args`.
This function will choose the correct version of `objc_msgSend` based on the
return type. For more information, see Apple's documenation:
https://developer.apple.com/library/mac/documentation/Cocoa/Reference/ObjCRuntimeRef/index.html#//apple_ref/doc/uid/TP40001418-CH1g-88778

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
pub unsafe fn send_message<T, A, R>(obj: &T, sel: Sel, args: A) -> R
        where T: ToMessage, A: MessageArguments {
    args.send(obj, sel)
}

/// Sends a message to the superclass of an instance of a class.
pub unsafe fn send_super_message<T, A, R>(
        obj: &T, superclass: &Class, sel: Sel, args: A) -> R
        where T: ToMessage, A: MessageArguments {
    args.send_super(obj, superclass, sel)
}

#[cfg(test)]
mod tests {
    use runtime::Object;
    use test_utils;
    use super::{send_message, send_super_message};

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

    #[test]
    fn test_send_message_super() {
        let obj = test_utils::custom_subclass_object();
        let superclass = test_utils::custom_class();
        unsafe {
            let _: () = send_message(&obj, sel!(setFoo:), (4u32,));
            assert!(send_super_message(&obj, superclass, sel!(foo), ()) == 4u32);
            // The subclass is overriden to return foo + 2
            assert!(send_message(&obj, sel!(foo), ()) == 6u32);
        }
    }
}

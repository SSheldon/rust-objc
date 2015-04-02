use std::marker::PhantomFn;
use std::mem;

use block::Block;
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

unsafe impl<A, R> Message for Block<A, R> { }

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
    /// The correct version of `objc_msgSend` will be chosen based on the
    /// return type. For more information, see Apple's documenation:
    /// https://developer.apple.com/library/mac/documentation/Cocoa/Reference/ObjCRuntimeRef/index.html#//apple_ref/doc/uid/TP40001418-CH1g-88778
    ///
    /// It is recommended to use the `msg_send!` macro rather than calling this
    /// method directly.
    unsafe fn send<T, R>(self, obj: *mut T, sel: Sel) -> R where T: Message;

    /// Sends a message to the superclass of an instance of a class with self
    /// as the arguments.
    unsafe fn send_super<T, R>(self, obj: *mut T, superclass: &Class, sel: Sel) -> R
            where T: Message;
}

macro_rules! message_args_impl {
    ($($a:ident : $t:ident),*) => (
        impl<$($t),*> MessageArguments for ($($t,)*) {
            unsafe fn send<T, R>(self, obj: *mut T, sel: Sel) -> R where T: Message {
                let msg_send_fn = msg_send_fn::<R>();
                let msg_send_fn: unsafe extern fn(*mut Object, Sel $(, $t)*) -> R =
                    mem::transmute(msg_send_fn);
                let ($($a,)*) = self;
                msg_send_fn(obj as *mut Object, sel $(, $a)*)
            }

            unsafe fn send_super<T, R>(self, obj: *mut T, superclass: &Class, sel: Sel) -> R
                    where T: Message {
                let msg_send_fn = msg_send_super_fn::<R>();
                let msg_send_fn: unsafe extern fn(*const Super, Sel $(, $t)*) -> R =
                    mem::transmute(msg_send_fn);
                let sup = Super { receiver: obj as *mut Object, superclass: superclass };
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

#[cfg(test)]
mod tests {
    use std::ptr;
    use runtime::Object;
    use test_utils;

    #[test]
    fn test_send_message() {
        let obj = test_utils::sample_object();
        let result: *const Object = unsafe {
            msg_send![obj, self]
        };
        assert!(result == &*obj);
    }

    #[test]
    fn test_send_message_stret() {
        let obj = test_utils::custom_object();
        let result: test_utils::CustomStruct = unsafe {
            msg_send![obj, customStruct]
        };
        let expected = test_utils::CustomStruct { a: 1, b:2, c: 3, d: 4 };
        assert!(result == expected);
    }

    #[test]
    fn test_send_message_nil() {
        let nil: *mut Object = ptr::null_mut();
        let result: usize = unsafe {
            msg_send![nil, hash]
        };
        assert!(result == 0);

        let result: *mut Object = unsafe {
            msg_send![nil, description]
        };
        assert!(result.is_null());

        let result: f64 = unsafe {
            msg_send![nil, doubleValue]
        };
        assert!(result == 0.0);
    }

    #[test]
    fn test_send_message_super() {
        let obj = test_utils::custom_subclass_object();
        let superclass = test_utils::custom_class();
        unsafe {
            let _: () = msg_send![obj, setFoo:4u32];
            let foo: u32 = msg_send![super(obj, superclass), foo];
            assert!(foo == 4);

            // The subclass is overriden to return foo + 2
            let foo: u32 = msg_send![obj, foo];
            assert!(foo == 6);
        }
    }
}

use std::any::Any;
use std::mem;

use runtime::{Class, Object, Sel, Super, self};

/// Types that may be sent Objective-C messages.
/// For example: objects, classes, and blocks.
pub unsafe trait Message { }

unsafe impl Message for Object { }

unsafe impl Message for Class { }

#[cfg(target_arch = "x86")]
#[path = "x86.rs"]
mod platform;
#[cfg(target_arch = "x86_64")]
#[path = "x86_64.rs"]
mod platform;
#[cfg(target_arch = "arm")]
#[path = "arm.rs"]
mod platform;
#[cfg(all(target_arch = "aarch64", not(feature = "gnustep")))]
#[path = "arm64.rs"]
mod platform;

use self::platform::{msg_send_fn, msg_send_super_fn};

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
    unsafe fn send<T, R>(self, obj: *mut T, sel: Sel) -> R
            where T: Message, R: Any;

    /// Sends a message to the superclass of an instance of a class with self
    /// as the arguments.
    unsafe fn send_super<T, R>(self, obj: *mut T, superclass: &Class, sel: Sel) -> R
            where T: Message, R: Any;
}

macro_rules! message_args_impl {
    ($($a:ident : $t:ident),*) => (
        impl<$($t),*> MessageArguments for ($($t,)*) {
            #[cfg(any(not(feature="gnustep"),
                      any(target_arch = "arm",
                          target_arch = "x86",
                          target_arch = "x86_64")))]
            unsafe fn send<T, R>(self, obj: *mut T, sel: Sel) -> R
                    where T: Message, R: Any {
                let msg_send_fn = msg_send_fn::<R>();
                let msg_send_fn: unsafe extern fn(*mut Object, Sel $(, $t)*) -> R =
                    mem::transmute(msg_send_fn);
                let ($($a,)*) = self;
                objc_try!({
                    msg_send_fn(obj as *mut Object, sel $(, $a)*)
                })
            }

            #[cfg(all(feature="gnustep",
                      not(any(target_arch = "arm",
                              target_arch = "x86",
                              target_arch = "x86_64"))))]
            unsafe fn send<T, R>(self, obj: *mut T, sel: Sel) -> R
                    where T: Message, R: Any {
                let mut receiver = obj as *mut Object;
                let nil: *mut Object = ::std::ptr::null_mut();
                let ref slot = *runtime::objc_msg_lookup_sender(&mut receiver as *mut *mut Object, sel, nil);
                let imp_fn = slot.method;
                let imp_fn: unsafe extern fn(*mut Object, Sel $(, $t)*) -> R =
                    mem::transmute(imp_fn);
                let ($($a,)*) = self;
                objc_try!({
                    imp_fn(receiver as *mut Object, sel $(, $a)*)
                })
            }

            #[cfg(not(feature="gnustep"))]
            unsafe fn send_super<T, R>(self, obj: *mut T, superclass: &Class, sel: Sel) -> R
                    where T: Message, R: Any {
                let msg_send_fn = msg_send_super_fn::<R>();
                let msg_send_fn: unsafe extern fn(*const Super, Sel $(, $t)*) -> R =
                    mem::transmute(msg_send_fn);
                let sup = Super { receiver: obj as *mut Object, superclass: superclass };
                let ($($a,)*) = self;
                objc_try!({
                    msg_send_fn(&sup, sel $(, $a)*)
                })
            }

            #[cfg(feature="gnustep")]
            unsafe fn send_super<T, R>(self, obj: *mut T, superclass: &Class, sel: Sel) -> R
                    where T: Message, R: Any {
                let sup = Super { receiver: obj as *mut Object, superclass: superclass };
                let ref slot = *runtime::objc_slot_lookup_super(&sup, sel);
                let imp_fn = slot.method;
                let imp_fn: unsafe extern fn(*mut Object, Sel $(, $t)*) -> R =
                    mem::transmute(imp_fn);
                let ($($a,)*) = self;
                objc_try!({
                    imp_fn(obj as *mut Object, sel $(, $a)*)
                })
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

#[doc(hidden)]
#[inline(always)]
#[cfg(not(feature = "verify_message"))]
pub unsafe fn send_message<T, A, R>(obj: *const T, sel: Sel, args: A)
        -> Result<R, String>
        where T: Message, A: MessageArguments, R: Any {
    Ok(args.send(obj as *mut T, sel))
}

#[doc(hidden)]
#[inline(always)]
#[cfg(feature = "verify_message")]
pub unsafe fn send_message<T, A, R>(obj: *const T, sel: Sel, args: A)
        -> Result<R, String>
        where T: Message, A: MessageArguments + ::verify::EncodeArguments,
        R: Any + ::Encode {
    use verify::verify_message_signature;

    let cls = if obj.is_null() {
        return Err(format!("Messaging {:?} to nil", sel));
    } else {
        (*(obj as *const Object)).class()
    };

    verify_message_signature::<A, R>(cls, sel).map(|_| {
        args.send(obj as *mut T, sel)
    })
}

#[doc(hidden)]
#[inline(always)]
#[cfg(not(feature = "verify_message"))]
pub unsafe fn send_super_message<T, A, R>(obj: *const T, superclass: &Class,
        sel: Sel, args: A) -> Result<R, String>
        where T: Message, A: MessageArguments, R: Any {
    Ok(args.send_super(obj as *mut T, superclass, sel))
}

#[doc(hidden)]
#[inline(always)]
#[cfg(feature = "verify_message")]
pub unsafe fn send_super_message<T, A, R>(obj: *const T, superclass: &Class,
        sel: Sel, args: A) -> Result<R, String>
        where T: Message, A: MessageArguments + ::verify::EncodeArguments,
        R: Any + ::Encode {
    use verify::verify_message_signature;

    if obj.is_null() {
        return Err(format!("Messaging {:?} to nil", sel));
    }

    verify_message_signature::<A, R>(superclass, sel).map(|_| {
        args.send_super(obj as *mut T, superclass, sel)
    })
}

#[cfg(test)]
mod tests {
    use runtime::Object;
    use test_utils;

    #[test]
    fn test_send_message() {
        let obj = test_utils::custom_object();
        let result: u32 = unsafe {
            let _: () = msg_send![obj, setFoo:4u32];
            msg_send![obj, foo]
        };
        assert!(result == 4);
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
        let nil: *mut Object = ::std::ptr::null_mut();
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

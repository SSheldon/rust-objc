use std::mem;

use runtime::{Class, Method, Object, Sel, self};
use {encode, Encode};

/*
 The Sized bound on Message is unfortunate; ideally, objc objects would not be
 treated as Sized. However, rust won't allow casting a dynamically-sized type
 pointer to an Object pointer, because dynamically-sized types can have fat
 pointers (two words) instead of real pointers.
 */
/// Types that may be sent Objective-C messages.
/// For example: objects, classes, and blocks.
pub unsafe trait Message: Sized { }

unsafe impl Message for Object { }

unsafe impl Message for Class { }

/// A trait for converting to a pointer to a type that may be sent Objective-C
/// messages.
pub trait ToMessage {
    type Target: Message;

    fn as_ptr(&self) -> *mut Self::Target;

    fn is_nil(&self) -> bool {
        self.as_ptr().is_null()
    }
}

impl<T> ToMessage for *const T where T: Message {
    type Target = T;

    fn as_ptr(&self) -> *mut T {
        *self as *mut T
    }
}

impl<T> ToMessage for *mut T where T: Message {
    type Target = T;

    fn as_ptr(&self) -> *mut T {
        *self
    }
}

impl<'a, T> ToMessage for &'a T where T: Message {
    type Target = T;

    fn as_ptr(&self) -> *mut T {
        *self as *const T as *mut T
    }
}

impl<'a, T> ToMessage for &'a mut T where T: Message {
    type Target = T;

    fn as_ptr(&self) -> *mut T {
        *self
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

pub trait MessageArguments {
    unsafe fn send<T, R>(self, obj: &T, sel: Sel) -> R where T: ToMessage;
}

macro_rules! message_args_impl {
    ($($a:ident : $t:ident),*) => (
        impl<$($t),*> MessageArguments for ($($t,)*) {
            unsafe fn send<T, R>(self, obj: &T, sel: Sel) -> R where T: ToMessage {
                let msg_send_fn = msg_send_fn::<R>();
                let msg_send_fn: unsafe extern fn(*mut Object, Sel $(, $t)*) -> R =
                    mem::transmute(msg_send_fn);
                let obj_ptr = obj.as_ptr() as *mut Object;
                let ($($a,)*) = self;
                msg_send_fn(obj_ptr, sel $(, $a)*)
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
fn verify_message_signature<T, A, R>(obj: Option<&T>, sel: Sel, _args: &A) ->
        Result<(), String> where T: Message, A: MessageArguments, R: Encode {
    let obj = match obj {
        Some(obj) => obj,
        None => return Err(format!("Messaging {:?} to nil", sel)),
    };
    let cls = unsafe {
        let obj = &*(obj as *const _ as *const Object);
        obj.class()
    };
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

#[cfg(all(not(ndebug), feature = "verify_message_encode"))]
pub unsafe fn send_message<T, A, R>(obj: &T, sel: Sel, args: A) -> R
        where T: ToMessage, A: MessageArguments, R: Encode {
    let obj_ref = obj.as_ptr().as_ref();
    match verify_message_signature::<_, _, R>(obj_ref, sel, &args) {
        Err(s) => panic!("Verify message failed: {}", s),
        Ok(_) => args.send(obj, sel),
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

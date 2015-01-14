use std::ptr;

use runtime::{Class, Object};

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

impl<'a, T> ToMessage for Option<&'a T> where T: Message {
    type Target = T;

    fn as_ptr(&self) -> *mut T {
        match *self {
            None => ptr::null_mut(),
            Some(ref obj) => obj.as_ptr(),
        }
    }
}

impl<'a, T> ToMessage for Option<&'a mut T> where T: Message {
    type Target = T;

    fn as_ptr(&self) -> *mut T {
        match *self {
            None => ptr::null_mut(),
            Some(ref obj) => obj.as_ptr(),
        }
    }
}

/// Converts to an Object pointer; this function is mainly used by the
/// `msg_send!` macro.
pub fn to_obj_ptr<M>(obj_ref: &M) -> *mut Object where M: ToMessage {
    obj_ref.as_ptr() as *mut Object
}

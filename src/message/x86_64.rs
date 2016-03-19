use std::mem;

use runtime::{Object, Imp, Sel};
use super::Super;

pub fn msg_send_fn<R>(obj: *mut Object, _: Sel) -> (Imp, *mut Object) {
    // If the size of an object is larger than two eightbytes, it has class MEMORY.
    // If the type has class MEMORY, then the caller provides space for the return
    // value and passes the address of this storage.
    // http://people.freebsd.org/~obrien/amd64-elf-abi.pdf

    extern {
        fn objc_msgSend(obj: *mut Object, op: Sel, ...) -> *mut Object;
        fn objc_msgSend_stret(obj: *mut Object, op: Sel, ...);
    }

    let msg_fn = if mem::size_of::<R>() <= 16 {
        unsafe { mem::transmute(objc_msgSend) }
    } else {
        unsafe { mem::transmute(objc_msgSend_stret) }
    };

    (msg_fn, obj)
}

pub fn msg_send_super_fn<R>(sup: &Super, _: Sel) -> (Imp, *mut Object) {
    extern {
        fn objc_msgSendSuper(sup: *const Super, op: Sel, ...) -> *mut Object;
        fn objc_msgSendSuper_stret(sup: *const Super, op: Sel, ... );
    }

    let msg_fn = if mem::size_of::<R>() <= 16 {
        unsafe { mem::transmute(objc_msgSendSuper) }
    } else {
        unsafe { mem::transmute(objc_msgSendSuper_stret) }
    };

    (msg_fn, sup as *const Super as *mut Object)
}

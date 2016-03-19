use std::mem;

use runtime::{Object, Imp, Sel, Super};

pub fn msg_send_fn<R>(obj: *mut Object, _: Sel) -> (Imp, *mut Object) {
    // stret is not even available in arm64.
    // https://twitter.com/gparker/status/378079715824660480

    extern {
        fn objc_msgSend(obj: *mut Object, op: Sel, ...) -> *mut Object;
    }

    let msg_fn = unsafe { mem::transmute(objc_msgSend) };
    (msg_fn, obj)
}

pub fn msg_send_super_fn<R>(sup: &Super, _: Sel) -> (Imp, *mut Object) {
    extern {
        fn objc_msgSendSuper(sup: *const Super, op: Sel, ...) -> *mut Object;
    }

    let msg_fn = unsafe { mem::transmute(objc_msgSendSuper) };
    (msg_fn, sup as *const Super as *mut Object)
}

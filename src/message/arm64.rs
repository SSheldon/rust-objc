use std::mem;

use runtime::{Object, Imp, Sel, Super, self};

pub fn msg_send_fn<R>(obj: *mut Object, _: Sel) -> (Imp, *mut Object) {
    // stret is not even available in arm64.
    // https://twitter.com/gparker/status/378079715824660480

    let msg_fn = unsafe { mem::transmute(runtime::objc_msgSend) };
    (msg_fn, obj)
}

pub fn msg_send_super_fn<R>(sup: &Super, _: Sel) -> (Imp, *mut Object) {
    let msg_fn = unsafe { mem::transmute(runtime::objc_msgSendSuper) };
    (msg_fn, sup as *const Super as *mut Object)
}

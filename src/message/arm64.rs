use std::any::Any;
use std::mem;

use runtime::{Object, Sel, Super, self};

pub fn msg_send_fn<R>() -> unsafe extern fn(*mut Object, Sel, ...) -> R {
    // stret is not even available in arm64.
    // https://twitter.com/gparker/status/378079715824660480

    unsafe { mem::transmute(runtime::objc_msgSend) }
}

pub fn msg_send_super_fn<R>() -> unsafe extern fn(*const Super, Sel, ...) -> R {
    unsafe { mem::transmute(runtime::objc_msgSendSuper) }
}

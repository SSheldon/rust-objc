use runtime::{Object, Imp, Sel};
use super::Super;

pub fn msg_send_fn<R>(obj: *mut Object, _: Sel) -> (Imp, *mut Object) {
    // stret is not even available in arm64.
    // https://twitter.com/gparker/status/378079715824660480

    extern {
        fn objc_msgSend();
    }

    (objc_msgSend, obj)
}

pub fn msg_send_super_fn<R>(sup: &Super, _: Sel) -> (Imp, *mut Object) {
    extern {
        fn objc_msgSendSuper();
    }

    (objc_msgSendSuper, sup as *const Super as *mut Object)
}

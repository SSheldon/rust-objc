use runtime::{Object, Imp, Sel};
use super::Super;

pub fn msg_send_fn<R>(obj: *mut Object, sel: Sel) -> (Imp, *mut Object) {
    extern {
        fn objc_msg_lookup(receiver: *mut Object, op: Sel) -> Imp;
    }

    let imp_fn = unsafe {
        objc_msg_lookup(obj, sel)
    };
    (imp_fn, obj)
}

pub fn msg_send_super_fn<R>(sup: &Super, sel: Sel) -> (Imp, *mut Object) {
    extern {
        fn objc_msg_lookup_super(sup: *const Super, sel: Sel) -> Imp;
    }

    let imp_fn = unsafe {
        objc_msg_lookup_super(sup, sel)
    };
    (imp_fn, sup.receiver)
}

use std::any::{Any, TypeId};
use std::mem;

use runtime::{Object, Imp, Sel};
use super::Super;

pub fn msg_send_fn<R: Any>(obj: *mut Object, _: Sel) -> (Imp, *mut Object) {
    // Double-word sized fundamental data types don't use stret,
    // but any composite type larger than 4 bytes does.
    // http://infocenter.arm.com/help/topic/com.arm.doc.ihi0042e/IHI0042E_aapcs.pdf

    extern {
        fn objc_msgSend(obj: *mut Object, op: Sel, ...) -> *mut Object;
        fn objc_msgSend_stret(obj: *mut Object, op: Sel, ...);
    }

    let type_id = TypeId::of::<R>();
    let msg_fn = if mem::size_of::<R>() <= 4 ||
            type_id == TypeId::of::<i64>() ||
            type_id == TypeId::of::<u64>() ||
            type_id == TypeId::of::<f64>() {
        unsafe { mem::transmute(objc_msgSend) }
    } else {
        unsafe { mem::transmute(objc_msgSend_stret) }
    };

    (msg_fn, obj)
}

pub fn msg_send_super_fn<R: Any>(sup: &Super, _: Sel) -> (Imp, *mut Object) {
    extern {
        fn objc_msgSendSuper(sup: *const Super, op: Sel, ...) -> *mut Object;
        fn objc_msgSendSuper_stret(sup: *const Super, op: Sel, ... );
    }

    let type_id = TypeId::of::<R>();
    let msg_fn = if mem::size_of::<R>() <= 4 ||
            type_id == TypeId::of::<i64>() ||
            type_id == TypeId::of::<u64>() ||
            type_id == TypeId::of::<f64>() {
        unsafe { mem::transmute(objc_msgSendSuper) }
    } else {
        unsafe { mem::transmute(objc_msgSendSuper_stret) }
    };

    (msg_fn, sup as *const Super as *mut Object)
}

use std::any::Any;
use std::mem;

use runtime::{Object, Sel, Super, self};

pub fn msg_send_fn<R: Any>() -> unsafe extern fn(*mut Object, Sel, ...) -> R {
    // Double-word sized fundamental data types don't use stret,
    // but any composite type larger than 4 bytes does.
    // http://infocenter.arm.com/help/topic/com.arm.doc.ihi0042e/IHI0042E_aapcs.pdf

    use std::any::TypeId;

    let type_id = TypeId::of::<R>();
    if mem::size_of::<R>() <= 4 ||
            type_id == TypeId::of::<i64>() ||
            type_id == TypeId::of::<u64>() ||
            type_id == TypeId::of::<f64>() {
        unsafe { mem::transmute(runtime::objc_msgSend) }
    } else {
        unsafe { mem::transmute(runtime::objc_msgSend_stret) }
    }
}

#[cfg(not(feature = "gnustep"))]
pub fn msg_send_super_fn<R: Any>() -> unsafe extern fn(*mut Object, Sel, ...) -> R {
    use std::any::TypeId;

    let type_id = TypeId::of::<R>();
    if mem::size_of::<R>() <= 4 ||
            type_id == TypeId::of::<i64>() ||
            type_id == TypeId::of::<u64>() ||
            type_id == TypeId::of::<f64>() {
        unsafe { mem::transmute(runtime::objc_msgSendSuper) }
    } else {
        unsafe { mem::transmute(runtime::objc_msgSendSuper_stret) }
    }
}

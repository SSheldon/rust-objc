use std::any::{Any, TypeId};
use std::mem;

use runtime::{Object, Imp, Sel, Super};

pub fn msg_send_fn<R: Any>(obj: *mut Object, _: Sel) -> (Imp, *mut Object) {
    // Structures 1 or 2 bytes in size are placed in EAX.
    // Structures 4 or 8 bytes in size are placed in: EAX and EDX.
    // Structures of other sizes are placed at the address supplied by the caller.
    // https://developer.apple.com/library/mac/documentation/DeveloperTools/Conceptual/LowLevelABI/130-IA-32_Function_Calling_Conventions/IA32.html

    extern {
        fn objc_msgSend(obj: *mut Object, op: Sel, ...) -> *mut Object;
        fn objc_msgSend_fpret(obj: *mut Object, op: Sel, ...) -> f64;
        fn objc_msgSend_stret(obj: *mut Object, op: Sel, ...);
    }

    let type_id = TypeId::of::<R>();
    let size = mem::size_of::<R>();
    let msg_fn = if type_id == TypeId::of::<f32>() ||
            type_id == TypeId::of::<f64>() {
        unsafe { mem::transmute(objc_msgSend_fpret) }
    } else if size == 0 || size == 1 || size == 2 || size == 4 || size == 8 {
        unsafe { mem::transmute(objc_msgSend) }
    } else {
        unsafe { mem::transmute(objc_msgSend_stret) }
    };

    (msg_fn, obj)
}

#[cfg(not(feature = "gnustep"))]
pub fn msg_send_super_fn<R: Any>(sup: &Super, _: Sel) -> (Imp, *mut Object) {
    extern {
        fn objc_msgSendSuper(sup: *const Super, op: Sel, ...) -> *mut Object;
        fn objc_msgSendSuper_stret(sup: *const Super, op: Sel, ... );
    }

    let size = mem::size_of::<R>();
    let msg_fn = if size == 0 || size == 1 || size == 2 || size == 4 || size == 8 {
        unsafe { mem::transmute(objc_msgSendSuper) }
    } else {
        unsafe { mem::transmute(objc_msgSendSuper_stret) }
    };

    (msg_fn, sup as *const Super as *mut Object)
}

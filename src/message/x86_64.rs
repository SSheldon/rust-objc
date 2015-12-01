use std::mem;

use runtime::{Object, Imp, Sel, Super, self};

pub fn msg_send_fn<R>(obj: *mut Object, _: Sel) -> (Imp, *mut Object) {
    // If the size of an object is larger than two eightbytes, it has class MEMORY.
    // If the type has class MEMORY, then the caller provides space for the return
    // value and passes the address of this storage.
    // http://people.freebsd.org/~obrien/amd64-elf-abi.pdf

    let msg_fn = if mem::size_of::<R>() <= 16 {
        unsafe { mem::transmute(runtime::objc_msgSend) }
    } else {
        unsafe { mem::transmute(runtime::objc_msgSend_stret) }
    };

    (msg_fn, obj)
}

#[cfg(not(feature = "gnustep"))]
pub fn msg_send_super_fn<R>(sup: &Super, _: Sel) -> (Imp, *mut Object) {
    let msg_fn = if mem::size_of::<R>() <= 16 {
        unsafe { mem::transmute(runtime::objc_msgSendSuper) }
    } else {
        unsafe { mem::transmute(runtime::objc_msgSendSuper_stret) }
    };

    (msg_fn, sup as *const Super as *mut Object)
}

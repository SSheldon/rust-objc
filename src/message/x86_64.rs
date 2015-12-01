use std::any::Any;
use std::mem;

use runtime::{Object, Sel, Super, self};

pub fn msg_send_fn<R>() -> unsafe extern fn(*mut Object, Sel, ...) -> R {
    // If the size of an object is larger than two eightbytes, it has class MEMORY.
    // If the type has class MEMORY, then the caller provides space for the return
    // value and passes the address of this storage.
    // http://people.freebsd.org/~obrien/amd64-elf-abi.pdf

    if mem::size_of::<R>() <= 16 {
        unsafe { mem::transmute(runtime::objc_msgSend) }
    } else {
        unsafe { mem::transmute(runtime::objc_msgSend_stret) }
    }
}

#[cfg(not(feature = "gnustep"))]
pub fn msg_send_super_fn<R>() -> unsafe extern fn(*const Super, Sel, ...) -> R {
    if mem::size_of::<R>() <= 16 {
        unsafe { mem::transmute(runtime::objc_msgSendSuper) }
    } else {
        unsafe { mem::transmute(runtime::objc_msgSendSuper_stret) }
    }
}

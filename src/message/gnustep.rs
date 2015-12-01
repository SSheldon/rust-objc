use runtime::{Object, Imp, Sel, Super, self};

#[cfg(any(target_arch = "arm",
          target_arch = "x86",
          target_arch = "x86_64"))]
pub use super::platform::msg_send_fn;

#[cfg(not(any(target_arch = "arm",
              target_arch = "x86",
              target_arch = "x86_64")))]
pub fn msg_send_fn<R>(obj: *mut Object, sel: Sel) -> (Imp, *mut Object) {
    let mut receiver = obj;
    let sender = ::std::ptr::null_mut();
    let slot = unsafe {
        &*runtime::objc_msg_lookup_sender(&mut receiver, sel, sender)
    };
    (slot.method, receiver)
}

pub fn msg_send_super_fn<R>(sup: &Super, sel: Sel) -> (Imp, *mut Object) {
    let slot = unsafe {
        &*runtime::objc_slot_lookup_super(sup, sel)
    };
    (slot.method, sup.receiver)
}

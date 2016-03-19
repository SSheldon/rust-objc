use std::os::raw::{c_char, c_int};

use runtime::{Object, Class, Imp, Sel, Super};

/// A structure describing a safely cacheable method implementation
/// in the GNUstep Objective-C runtime.
#[repr(C)]
struct Slot  {
    /// The class to which the slot is attached
    pub owner: *const Class,
    /// The class for which this slot was cached.
    pub cached_for: *mut Class,
    /// The type signature of the method
    pub types: *const c_char,
    /// The version of the method. Will change if overriden, invalidating
    /// the cache
    pub version: c_int,
    /// The implementation of the method
    pub method: Imp,
    /// The associated selector
    pub selector: Sel,
}

#[cfg(any(target_arch = "arm",
          target_arch = "x86",
          target_arch = "x86_64"))]
pub use super::platform::msg_send_fn;

#[cfg(not(any(target_arch = "arm",
              target_arch = "x86",
              target_arch = "x86_64")))]
pub fn msg_send_fn<R>(obj: *mut Object, sel: Sel) -> (Imp, *mut Object) {
    extern {
        fn objc_msg_lookup_sender(receiver: *mut *mut Object, selector: Sel, sender: *mut Object, ...) -> *mut Slot;
    }

    let mut receiver = obj;
    let sender = ::std::ptr::null_mut();
    let slot = unsafe {
        &*objc_msg_lookup_sender(&mut receiver, sel, sender)
    };
    (slot.method, receiver)
}

pub fn msg_send_super_fn<R>(sup: &Super, sel: Sel) -> (Imp, *mut Object) {
    extern {
        fn objc_slot_lookup_super(sup: *const Super, selector: Sel) -> *mut Slot;
    }

    let slot = unsafe {
        &*objc_slot_lookup_super(sup, sel)
    };
    (slot.method, sup.receiver)
}

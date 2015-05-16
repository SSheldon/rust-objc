use std::ops::Deref;

use runtime::Object;

pub struct StrongPtr(pub *mut Object);

impl Deref for StrongPtr {
    type Target = Object;

    fn deref(&self) -> &Object {
        unsafe { &*self.0 }
    }
}

impl Drop for StrongPtr {
    fn drop(&mut self) {
        let _: () = unsafe { msg_send![self.0, release] };
    }
}

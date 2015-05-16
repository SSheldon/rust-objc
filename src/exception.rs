use objc_exception;

use id::StrongPtr;
use runtime::Object;

pub unsafe fn try<F, R>(closure: F) -> Result<R, Option<StrongPtr>>
        where F: FnOnce() -> R {
    objc_exception::try(closure).map_err(|exception| {
        if exception.is_null() { None }
        else { Some(StrongPtr::new(exception as *mut Object)) }
    })
}

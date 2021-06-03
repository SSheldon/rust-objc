use core::mem;

use super::{Message, MessageArguments, MessageError, Super};
use crate::runtime::{Class, Imp, Object, Sel};

extern "C" {
    fn objc_msg_lookup(receiver: *mut Object, op: Sel) -> Imp;
    fn objc_msg_lookup_super(sup: *const Super, sel: Sel) -> Imp;
}

pub unsafe fn send_unverified<T, A, R>(obj: *const T, sel: Sel, args: A) -> Result<R, MessageError>
where
    T: Message,
    A: MessageArguments,
{
    if obj.is_null() {
        return mem::zeroed();
    }

    let receiver = obj as *mut T as *mut Object;
    let msg_send_fn = objc_msg_lookup(receiver, sel);
    objc_try!({ A::invoke(msg_send_fn, receiver, sel, args) })
}

pub unsafe fn send_super_unverified<T, A, R>(
    obj: *const T,
    superclass: &Class,
    sel: Sel,
    args: A,
) -> Result<R, MessageError>
where
    T: Message,
    A: MessageArguments,
{
    let receiver = obj as *mut T as *mut Object;
    let sup = Super {
        receiver: receiver,
        superclass: superclass,
    };
    let msg_send_fn = objc_msg_lookup_super(&sup, sel);
    objc_try!({ A::invoke(msg_send_fn, receiver, sel, args) })
}

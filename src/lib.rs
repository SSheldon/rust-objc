/*!
Objective-C Runtime bindings and wrapper for Rust.

# Messaging objects

Objective-C objects can be messaged using the [`msg_send!`](macro.msg_send!.html) macro:

```
# #[macro_use] extern crate objc;
# use objc::runtime::{BOOL, Class, Object};
# fn main() {
# unsafe {
let cls = Class::get("NSObject").unwrap();
let obj: *mut Object = msg_send![cls, new];
let hash: usize = msg_send![obj, hash];
let is_kind: BOOL = msg_send![obj, isKindOfClass:cls];
// Even void methods must have their return type annotated
let _: () = msg_send![obj, release];
# }
# }
```

# Reference counting

Objective-C objects are reference counted; to ensure that they are retained and
released at the proper times, we can use the [`Id`](struct.Id.html) struct.

To enforce aliasing rules, an `Id` can be either owned or shared; if it is
owned, meaning the `Id` is the only reference to the object, it can be mutably
dereferenced. An owned `Id` can be downgraded to a [`ShareId`](type.ShareId.html)
which can be cloned to allow multiple references.

Weak references may be created using the [`WeakId`](struct.WeakId.html) struct.

```
# #[macro_use] extern crate objc;
# use objc::runtime::{BOOL, Class, Object};
# use objc::{Id, WeakId};
# fn main() {
let cls = Class::get("NSObject").unwrap();
let obj: Id<Object> = unsafe {
    Id::from_retained_ptr(msg_send![cls, new])
};
// obj will be released when it goes out of scope

// share the object so we can clone it
let obj = obj.share();
let another_ref = obj.clone();
// dropping our other reference will decrement the retain count
drop(another_ref);

let weak = WeakId::new(&obj);
assert!(weak.load().is_some());
// After the object is deallocated, our weak pointer returns none
drop(obj);
assert!(weak.load().is_none());
# }
```

# Declaring classes

Objective-C classes can even be declared from Rust using the functionality of
the [`declare`](declare/index.html) module.
*/

#![crate_name = "objc"]
#![crate_type = "lib"]

#![warn(missing_docs)]

extern crate libc;
extern crate malloc_buf;

pub use id::{Id, Owned, Ownership, Shared, ShareId};
pub use encode::{Encode, Encoding};
pub use message::{Message, MessageArguments};
pub use weak::WeakId;

#[macro_use]
mod macros;

pub mod runtime;
mod id;
pub mod declare;
mod encode;
mod message;
mod weak;

#[cfg(test)]
mod test_utils;

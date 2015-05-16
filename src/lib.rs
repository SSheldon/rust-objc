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

# Declaring classes

Objective-C classes can even be declared from Rust using the functionality of
the [`declare`](declare/index.html) module.
*/

#![crate_name = "objc"]
#![crate_type = "lib"]

#![warn(missing_docs)]

extern crate libc;
extern crate malloc_buf;
#[cfg(feature = "exception")]
extern crate objc_exception;

pub use encode::{Encode, Encoding};
pub use message::{Message, MessageArguments};

#[macro_use]
mod macros;

pub mod runtime;
pub mod declare;
mod encode;
#[cfg(feature = "exception")]
mod exception;
#[cfg(any(test, feature = "exception"))]
mod id;
mod message;

#[cfg(test)]
mod test_utils;

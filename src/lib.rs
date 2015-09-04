/*!
Objective-C Runtime bindings and wrapper for Rust.

# Messaging objects

Objective-C objects can be messaged using the [`msg_send!`](macro.msg_send!.html) macro:

```no_run
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

# Exceptions

By default, if the `msg_send!` macro causes an exception to be thrown, this
will unwind into Rust resulting in unsafe, undefined behavior.
However, this crate has an `"exception"` feature which, when enabled, wraps
each `msg_send!` in a `@try`/`@catch` and panics if an exception is caught,
preventing Objective-C from unwinding into Rust.
*/

#![crate_name = "objc"]
#![crate_type = "lib"]

#![warn(missing_docs)]

extern crate malloc_buf;
#[cfg(feature = "exception")]
extern crate objc_exception;

pub use encode::{Encode, Encoding};
pub use message::{Message, MessageArguments};

pub use message::send_message as __send_message;
pub use message::send_super_message as __send_super_message;

#[macro_use]
mod macros;

pub mod runtime;
pub mod declare;
mod encode;
#[cfg(feature = "exception")]
mod exception;
mod id;
mod message;
mod verify;

#[cfg(test)]
mod test_utils;

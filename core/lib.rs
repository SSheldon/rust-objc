//! Objective-C Runtime bindings and wrapper for Rust.

#![crate_name = "objc"]
#![crate_type = "lib"]

#![warn(missing_docs)]

extern crate libc;
extern crate malloc_buf;

#[cfg(test)]
extern crate objc_test_utils;

pub use id::{Id, Owned, Ownership, Shared, ShareId};
pub use encode::{Encode, Encoding};
pub use message::{Message, MessageArguments};
pub use weak::WeakId;

#[macro_use]
mod macros;

pub mod runtime;
mod id;
pub mod block;
pub mod declare;
mod encode;
mod message;
mod weak;

#[cfg(test)]
mod test_utils;

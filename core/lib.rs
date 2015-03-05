//! Objective-C Runtime bindings and wrapper for Rust.

#![crate_name = "objc"]
#![crate_type = "lib"]

#![feature(std_misc, unsafe_destructor)]
#![warn(missing_docs)]

extern crate libc;
extern crate malloc_buf;

#[cfg(test)]
extern crate objc_test_utils;

pub use id::{Id, Owned, Ownership, Shared, ShareId};
pub use encode::{encode, Encode, EncodePtr};
pub use message::{send_message, send_super_message, Message, MessageArguments, ToMessage};
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

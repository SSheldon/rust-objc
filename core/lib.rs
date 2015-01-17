#![crate_name = "objc"]
#![crate_type = "lib"]

#![feature(unboxed_closures, unsafe_destructor)]
#![allow(unstable)]

extern crate libc;

#[cfg(test)]
extern crate objc_test_utils;

pub use id::{Id, IdSlice, IntoIdVector, Owned, Ownership, Shared, ShareId};
pub use declare::{ClassDecl, MethodDecl};
pub use encode::{encode, Encode, Encoding};
pub use message::{Message, ToMessage};
pub use weak::WeakId;

#[macro_use]
mod macros;

pub mod runtime;
mod id;
pub mod block;
mod declare;
mod encode;
mod message;
mod weak;

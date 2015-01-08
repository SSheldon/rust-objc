#![crate_name = "objc"]
#![crate_type = "lib"]

#![feature(unsafe_destructor)]

extern crate libc;

#[cfg(test)]
extern crate objc_test_utils;

pub use id::{Id, IdVector, IntoIdVector, Owned, Ownership, Shared, ShareId};
pub use declare::{ClassDecl, MethodDecl};
pub use encode::{encode, Encode, Encoding};
pub use message::{to_obj_ptr, Message, ToMessage};
pub use weak::WeakId;

mod macros;

pub mod runtime;
mod id;
pub mod block;
mod declare;
mod encode;
mod message;
mod weak;

// Shim to re-export under the objc:: path for macros
mod objc {
    pub use super::*;
}

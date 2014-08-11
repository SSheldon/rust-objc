#![crate_name = "objc"]
#![crate_type = "lib"]

#![feature(default_type_params, globs, macro_rules, unsafe_destructor)]

extern crate libc;

pub use id::{Id, IdVector, IntoIdVector};
pub use class_name::{class, ClassName};
pub use declare::{ClassDecl, MethodDecl};
pub use encode::{encode, Encode, Encoding};

mod macros;

pub mod runtime;
mod id;
mod class_name;
mod declare;
mod encode;
pub mod foundation;

// Shim to re-export under the objc:: path for macros
mod objc {
	pub use runtime;
	pub use foundation;
	pub use super::*;
}

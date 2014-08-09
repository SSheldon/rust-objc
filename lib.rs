#![crate_name = "objc"]
#![crate_type = "lib"]

#![feature(default_type_params, macro_rules, unsafe_destructor)]

extern crate libc;

pub use id::{Id, IdVector, IntoIdVector};
pub use class_name::{class, ClassName};
pub use declare::{ClassDecl, MethodDecl};

mod macros;

pub mod runtime;
mod id;
mod class_name;
mod declare;
pub mod foundation;

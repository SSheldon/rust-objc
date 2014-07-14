#![crate_id = "objc"]

#![feature(default_type_params, macro_rules, unsafe_destructor)]

pub use id::{class, ClassName, Id, IdVector};

mod macros;

pub mod runtime;
mod id;
pub mod foundation;

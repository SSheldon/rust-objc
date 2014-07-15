#![crate_id = "objc"]
#![crate_type = "lib"]

#![feature(default_type_params, macro_rules, unsafe_destructor)]

pub use id::{Id, IdVector};
pub use class_name::{class, ClassName};

mod macros;

pub mod runtime;
mod id;
mod class_name;
pub mod foundation;

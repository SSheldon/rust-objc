#[macro_use]
extern crate objc;

pub use objc::*;

#[path = "../src/id.rs"]
mod id;
#[path = "../src/test_utils.rs"]
mod test_utils;

mod tests;

pub use tests::TESTS;

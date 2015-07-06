extern crate libc;
#[macro_use]
extern crate objc;

pub use objc::*;

#[path = "../src/id.rs"]
mod id;
#[path = "../src/test_utils.rs"]
mod test_utils;

mod tests;

use libc::{c_char, size_t};
use tests::TESTS;

pub extern fn tests_count() -> size_t {
    TESTS.len() as size_t
}

pub extern fn test_name(i: size_t, len: &mut size_t) -> *const c_char {
    let (name, _) = TESTS[i as usize];
    *len = name.len() as size_t;
    name.as_ptr() as *const c_char
}

pub extern fn run_test(i: size_t) {
    let (_, test_fn) = TESTS[i as usize];
    test_fn();
}

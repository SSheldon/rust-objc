#[macro_use]
extern crate objc;

pub use objc::*;

#[path = "../src/id.rs"]
mod id;
#[path = "../src/test_utils.rs"]
mod test_utils;

mod tests;

use std::os::raw::c_char;
use tests::TESTS;

#[no_mangle]
pub extern fn tests_count() -> usize {
    TESTS.len()
}

#[no_mangle]
pub extern fn test_name(i: usize, len: &mut usize) -> *const c_char {
    let (name, _) = TESTS[i as usize];
    *len = name.len();
    name.as_ptr() as *const c_char
}

#[no_mangle]
pub extern fn run_test(i: usize) {
    let (_, test_fn) = TESTS[i as usize];
    test_fn();
}

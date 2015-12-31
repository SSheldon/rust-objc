#![cfg_attr(not(feature = "verify_message"), allow(dead_code))]

use runtime::{Class, Sel};
use {Encode, EncodeArguments};

pub fn verify_message_signature<A, R>(cls: &Class, sel: Sel) -> Result<(), String>
        where A: EncodeArguments, R: Encode {
    let method = match cls.instance_method(sel) {
        Some(method) => method,
        None => return Err(format!("Method {:?} not found on class {:?}",
            sel, cls)),
    };

    let ret = R::encode();
    let expected_ret = method.return_type();
    if ret != expected_ret {
        return Err(format!("Return type code {:?} does not match expected {:?} for method {:?}",
            ret, expected_ret, method.name()));
    }

    let args = A::encodings();
    let args = args.as_ref();
    let count = args.len();
    let expected_count = method.arguments_count();
    if count != expected_count {
        return Err(format!("Method {:?} accepts {} arguments, but {} were given",
            method.name(), expected_count, count));
    }

    for (i, arg) in args.iter().enumerate() {
        let expected = method.argument_type(i).unwrap();
        if *arg != expected {
            return Err(format!("Method {:?} expected argument at index {} with type code {:?} but was given {:?}",
                method.name(), i, expected, arg));
        }
    }

    Ok(())
}

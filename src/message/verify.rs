use runtime::{Class, Method, Sel};
use {Encode, EncodeArguments, Encoding};
use super::MessageError;

pub fn verify_message_signature<A, R>(cls: &Class, sel: Sel)
        -> Result<(), MessageError>
        where A: EncodeArguments, R: Encode {
    let method = try!(verify_selector(cls, sel));

    let ret = R::encode();
    try!(verify_return(method, Some(&ret)));

    let args = A::encodings();
    verify_arguments(method, args.as_ref().iter().map(|a| Some(a)))
}

pub fn verify_selector(cls: &Class, sel: Sel) -> Result<&Method, MessageError> {
    match cls.instance_method(sel) {
        Some(method) => Ok(method),
        None => Err(MessageError(
            format!("Method {:?} not found on class {:?}",
                sel, cls)
        )),
    }
}

pub fn verify_return(method: &Method, ret: Option<&Encoding>)
        -> Result<(), MessageError> {
    let expected_ret = method.return_type();
    match ret {
        Some(ret) if *ret != expected_ret => Err(MessageError(
            format!("Return type code {:?} does not match expected {:?} for method {:?}",
                ret, expected_ret, method.name())
        )),
        _ => Ok(())
    }
}

pub fn verify_arguments<'a, I>(method: &Method, args: I)
        -> Result<(), MessageError>
        where I: Iterator<Item=Option<&'a Encoding>> {
    let expected_count = method.arguments_count();

    let mut i = 2;
    for arg in args {
        match (arg, method.argument_type(i)) {
            (Some(arg), Some(ref expected)) if arg != expected => {
                return Err(MessageError(
                    format!("Method {:?} expected argument at index {} with type code {:?} but was given {:?}",
                        method.name(), i, expected, arg)
                ));
            }
            _ => (),
        }
        i += 1;
    }

    if i == expected_count {
        Ok(())
    } else {
        Err(MessageError(
            format!("Method {:?} accepts {} arguments, but {} were given",
                method.name(), expected_count, i)
        ))
    }
}

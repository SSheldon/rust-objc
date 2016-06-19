use std::fmt;

use crate::runtime::{Class, Method, Sel};
use crate::Encoding;

pub enum VerificationError<'a> {
    NilReceiver(Sel),
    MethodNotFound(&'a Class, Sel),
    MismatchedReturn(&'a Method, Encoding<'static>),
    MismatchedArgumentsCount(&'a Method, usize),
    MismatchedArgument(&'a Method, usize, Encoding<'static>),
}

impl<'a> fmt::Display for VerificationError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            VerificationError::NilReceiver(sel) => {
                write!(f, "Messsaging {:?} to nil", sel)
            }
            VerificationError::MethodNotFound(cls, sel) => {
                write!(f, "Method {:?} not found on class {:?}", sel, cls)
            }
            VerificationError::MismatchedReturn(method, ret) => {
                let expected_ret = method.return_type();
                write!(f, "Return type code {} does not match expected {} for method {:?}",
                    ret, expected_ret, method.name())
            }
            VerificationError::MismatchedArgumentsCount(method, count) => {
                let expected_count = method.arguments_count();
                write!(f, "Method {:?} accepts {} arguments, but {} were given",
                    method.name(), expected_count, count)
            }
            VerificationError::MismatchedArgument(method, i, arg) => {
                let expected = method.argument_type(i).unwrap();
                write!(f, "Method {:?} expected argument at index {} with type code {} but was given {}",
                    method.name(), i, expected, arg)
            }
        }
    }
}

#[cfg(feature = "verify_message")]
pub fn verify_message_signature<A, R>(cls: &Class, sel: Sel)
        -> Result<(), super::MessageError>
        where A: crate::MessageArguments {
    let method = verify_selector(cls, sel)?;

    let ret = crate::encode::maybe_encode::<R>();
    verify_return(method, ret)?;

    A::verify(method)
}

pub fn verify_selector(cls: &Class, sel: Sel)
-> Result<&Method, VerificationError> {
    match cls.instance_method(sel) {
        Some(method) => Ok(method),
        None => Err(VerificationError::MethodNotFound(cls, sel)),
    }
}

pub fn verify_return<'a>(method: &'a Method, ret: Option<Encoding<'static>>)
-> Result<(), VerificationError<'a>> {
    let expected_ret = method.return_type();
    match ret {
        Some(ret) if ret != *expected_ret =>
            Err(VerificationError::MismatchedReturn(method, ret)),
        _ => Ok(())
    }
}

pub fn verify_arguments<I>(method: &Method, args: I)
-> Result<(), VerificationError>
where I: Iterator<Item=Option<Encoding<'static>>> {
    let expected_count = method.arguments_count();

    let mut i = 2;
    for arg in args {
        match (arg, method.argument_type(i)) {
            (Some(arg), Some(expected)) if arg != *expected => {
                return Err(VerificationError::MismatchedArgument(method, i, arg));
            }
            _ => (),
        }
        i += 1;
    }

    if i == expected_count {
        Ok(())
    } else {
        Err(VerificationError::MismatchedArgumentsCount(method, i))
    }
}

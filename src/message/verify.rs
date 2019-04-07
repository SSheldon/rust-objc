use crate::runtime::{Class, Object, Sel};
use crate::{Encode, EncodeArguments};
use super::MessageError;

pub fn verify_message_signature<A, R>(cls: &Class, sel: Sel)
        -> Result<(), MessageError>
        where A: EncodeArguments, R: Encode {
    let method = match cls.instance_method(sel) {
        Some(method) => method,
        None => return Err(MessageError(
            format!("Method {:?} not found on class {:?}",
                sel, cls)
        )),
    };

    let ret = R::ENCODING;
    let expected_ret = method.return_type();
    if ret != *expected_ret {
        return Err(MessageError(
            format!("Return type code {} does not match expected {} for method {:?}",
                ret, expected_ret, method.name())
        ));
    }

    let self_and_cmd = [<*mut Object>::ENCODING, Sel::ENCODING];
    let args = A::ENCODINGS;

    let count = self_and_cmd.len() + args.len();
    let expected_count = method.arguments_count();
    if count != expected_count {
        return Err(MessageError(
            format!("Method {:?} accepts {} arguments, but {} were given",
                method.name(), expected_count, count)
        ));
    }

    for (i, arg) in self_and_cmd.iter().chain(args).enumerate() {
        let expected = method.argument_type(i).unwrap();
        if *arg != *expected {
            return Err(MessageError(
                format!("Method {:?} expected argument at index {} with type code {:?} but was given {:?}",
                    method.name(), i, expected, arg)
            ));
        }
    }

    Ok(())
}

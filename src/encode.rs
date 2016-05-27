use crate::runtime::{Class, Object, Sel};
use crate::{Encode, Encoding};

unsafe impl Encode for Sel {
    const ENCODING: Encoding<'static> = Encoding::Sel;
}

unsafe impl<'a> Encode for &'a Object {
    const ENCODING: Encoding<'static> = Encoding::Object;
}

unsafe impl<'a> Encode for &'a mut Object {
    const ENCODING: Encoding<'static> = Encoding::Object;
}

unsafe impl<'a> Encode for &'a Class {
    const ENCODING: Encoding<'static> = Encoding::Class;
}

unsafe impl<'a> Encode for &'a mut Class {
    const ENCODING: Encoding<'static> = Encoding::Class;
}

/// Types that represent a group of arguments, where each has an Objective-C
/// type encoding.
pub trait EncodeArguments {
    /// The type as which the encodings for Self will be returned.
    const ENCODINGS: &'static [Encoding<'static>];
}

macro_rules! encode_args_impl {
    ($($t:ident),*) => (
        impl<$($t: Encode),*> EncodeArguments for ($($t,)*) {
            const ENCODINGS: &'static [Encoding<'static>] = &[
                $($t::ENCODING),*
            ];
        }
    );
}

encode_args_impl!();
encode_args_impl!(A);
encode_args_impl!(A, B);
encode_args_impl!(A, B, C);
encode_args_impl!(A, B, C, D);
encode_args_impl!(A, B, C, D, E);
encode_args_impl!(A, B, C, D, E, F);
encode_args_impl!(A, B, C, D, E, F, G);
encode_args_impl!(A, B, C, D, E, F, G, H);
encode_args_impl!(A, B, C, D, E, F, G, H, I);
encode_args_impl!(A, B, C, D, E, F, G, H, I, J);
encode_args_impl!(A, B, C, D, E, F, G, H, I, J, K);
encode_args_impl!(A, B, C, D, E, F, G, H, I, J, K, L);

#[cfg(feature = "verify_message")]
pub fn maybe_encode<T>() -> Option<Encoding<'static>> {
    trait MaybeEncode {
        fn maybe_encode() -> Option<Encoding<'static>>;
    }

    impl<T> MaybeEncode for T {
        default fn maybe_encode() -> Option<Encoding<'static>> {
            None
        }
    }

    impl<T: Encode> MaybeEncode for T {
        fn maybe_encode() -> Option<Encoding<'static>> {
            Some(T::ENCODING)
        }
    }

    T::maybe_encode()
}

#[cfg(test)]
mod tests {
    use objc_encode::Encode;
    use crate::runtime::{Class, Object, Sel};

    #[test]
    fn test_encode() {
        assert!(<&Object>::ENCODING.to_string() == "@");
        assert!(<*mut Object>::ENCODING.to_string() == "@");
        assert!(<&Class>::ENCODING.to_string() == "#");
        assert!(Sel::ENCODING.to_string() == ":");
    }
}

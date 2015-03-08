use std::marker::PhantomFn;
use libc::{c_char, c_void};

use block::Block;
use runtime::{Class, Object, Sel};

/// Types that have an Objective-C type encoding.
///
/// For more information, see Apple's documentation:
/// https://developer.apple.com/library/mac/documentation/Cocoa/Conceptual/ObjCRuntimeGuide/Articles/ocrtTypeEncodings.html
pub trait Encode : PhantomFn<Self> {
    /// Returns the encoding for Self.
    fn code() -> &'static str;
}

macro_rules! encode_impls {
    ($($t:ty : $s:expr,)*) => ($(
        impl Encode for $t {
            fn code() -> &'static str { $s }
        }
    )*);
}

encode_impls!(
    i8: "c",
    i16: "s",
    i32: "i",
    i64: "q",
    u8: "C",
    u16: "S",
    u32: "I",
    u64: "Q",
    f32: "f",
    f64: "d",
    bool: "B",
    (): "v",
    *mut c_char: "*",
    *const c_char: "r*",
    *mut c_void: "^v",
    *const c_void: "r^v",
    Sel: ":",
);

impl Encode for isize {
    #[cfg(target_pointer_width = "32")]
    fn code() -> &'static str { i32::code() }

    #[cfg(target_pointer_width = "64")]
    fn code() -> &'static str { i64::code() }
}

impl Encode for usize {
    #[cfg(target_pointer_width = "32")]
    fn code() -> &'static str { u32::code() }

    #[cfg(target_pointer_width = "64")]
    fn code() -> &'static str { u64::code() }
}

macro_rules! encode_message_impl {
    ($code:expr, $name:ident) => (
        encode_message_impl!($code, $name,);
    );
    ($code:expr, $name:ident, $($t:ident),*) => (
        impl<'a $(, $t)*> $crate::Encode for &'a $name<$($t),*> {
            fn code() -> &'static str { $code }
        }

        impl<'a $(, $t)*> $crate::Encode for &'a mut $name<$($t),*> {
            fn code() -> &'static str { $code }
        }

        impl<'a $(, $t)*> $crate::Encode for Option<&'a $name<$($t),*>> {
            fn code() -> &'static str { $code }
        }

        impl<'a $(, $t)*> $crate::Encode for Option<&'a mut $name<$($t),*>> {
            fn code() -> &'static str { $code }
        }

        impl<$($t),*> $crate::Encode for *const $name<$($t),*> {
            fn code() -> &'static str { $code }
        }

        impl<$($t),*> $crate::Encode for *mut $name<$($t),*> {
            fn code() -> &'static str { $code }
        }
    );
}

encode_message_impl!("@", Object);

encode_message_impl!("#", Class);

encode_message_impl!("@?", Block, A, R);

/// Returns the Objective-C type encoding for a type.
pub fn encode<T>() -> &'static str where T: Encode {
    T::code()
}

#[cfg(test)]
mod tests {
    use runtime::{Class, Object, Sel};
    use super::encode;

    #[test]
    fn test_encode() {
        assert!(encode::<u32>() == "I");
        assert!(encode::<()>() == "v");
        assert!(encode::<&Object>() == "@");
        assert!(encode::<*mut Object>() == "@");
        assert!(encode::<&Class>() == "#");
        assert!(encode::<Sel>() == ":");
    }
}

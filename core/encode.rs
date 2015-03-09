use std::marker::PhantomFn;
use libc::{c_char, c_void};

use block::Block;
use runtime::{Class, Object, Sel};

pub struct Encoding {
    code: &'static str,
}

macro_rules! static_encoding {
    ($s:expr) => (Encoding { code: concat!($s, "\0") });
}

impl Encoding {
    pub fn as_str(&self) -> &str {
        &self.code[..self.code.len() - 1]
    }

    pub fn as_ptr(&self) -> *const c_char {
        self.as_str().as_ptr() as *const c_char
    }

    pub fn unknown() -> Encoding { static_encoding!("?") }
}

impl PartialEq for Encoding {
    fn eq(&self, other: &Encoding) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<'a> PartialEq<&'a str> for Encoding {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_str() == *other
    }
}

/// Types that have an Objective-C type encoding.
///
/// For more information, see Apple's documentation:
/// https://developer.apple.com/library/mac/documentation/Cocoa/Conceptual/ObjCRuntimeGuide/Articles/ocrtTypeEncodings.html
pub trait Encode : PhantomFn<Self> {
    /// Returns the Objective-C type encoding for Self.
    fn encode() -> Encoding;
}

macro_rules! encode_impls {
    ($($t:ty : $s:expr,)*) => ($(
        impl Encode for $t {
            fn encode() -> Encoding { static_encoding!($s) }
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
    fn encode() -> Encoding { i32::encode() }

    #[cfg(target_pointer_width = "64")]
    fn encode() -> Encoding { i64::encode() }
}

impl Encode for usize {
    #[cfg(target_pointer_width = "32")]
    fn encode() -> Encoding { u32::encode() }

    #[cfg(target_pointer_width = "64")]
    fn encode() -> Encoding { u64::encode() }
}

macro_rules! encode_message_impl {
    ($code:expr, $name:ident) => (
        encode_message_impl!($code, $name,);
    );
    ($code:expr, $name:ident, $($t:ident),*) => (
        impl<'a $(, $t)*> $crate::Encode for &'a $name<$($t),*> {
            fn encode() -> Encoding { static_encoding!($code) }
        }

        impl<'a $(, $t)*> $crate::Encode for &'a mut $name<$($t),*> {
            fn encode() -> Encoding { static_encoding!($code) }
        }

        impl<'a $(, $t)*> $crate::Encode for Option<&'a $name<$($t),*>> {
            fn encode() -> Encoding { static_encoding!($code) }
        }

        impl<'a $(, $t)*> $crate::Encode for Option<&'a mut $name<$($t),*>> {
            fn encode() -> Encoding { static_encoding!($code) }
        }

        impl<$($t),*> $crate::Encode for *const $name<$($t),*> {
            fn encode() -> Encoding { static_encoding!($code) }
        }

        impl<$($t),*> $crate::Encode for *mut $name<$($t),*> {
            fn encode() -> Encoding { static_encoding!($code) }
        }
    );
}

encode_message_impl!("@", Object);

encode_message_impl!("#", Class);

encode_message_impl!("@?", Block, A, R);

#[cfg(test)]
mod tests {
    use runtime::{Class, Object, Sel};
    use super::Encode;

    #[test]
    fn test_encode() {
        assert!(u32::encode() == "I");
        assert!(<()>::encode() == "v");
        assert!(<&Object>::encode() == "@");
        assert!(<*mut Object>::encode() == "@");
        assert!(<&Class>::encode() == "#");
        assert!(Sel::encode() == ":");
    }
}

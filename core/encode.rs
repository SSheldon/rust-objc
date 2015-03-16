use std::fmt;
use std::marker::PhantomFn;
use libc::{c_char, c_void};

use block::Block;
use runtime::{Class, Object, Sel};

/// An Objective-C type encoding.
///
/// For more information, see Apple's documentation:
/// https://developer.apple.com/library/mac/documentation/Cocoa/Conceptual/ObjCRuntimeGuide/Articles/ocrtTypeEncodings.html
pub struct Encoding {
    code: &'static str,
}

impl Encoding {
    pub fn from_str(code: &'static str) -> Encoding {
        Encoding { code: code }
    }

    /// Returns self as a `str`.
    pub fn as_str(&self) -> &str {
        self.code
    }
}

impl Clone for Encoding {
    fn clone(&self) -> Encoding {
        Encoding { code: self.code }
    }
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

impl fmt::Debug for Encoding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Types that have an Objective-C type encoding.
///
/// Unsafe because Objective-C will make assumptions about the type (like its
/// size and alignment) from its encoding, so the implementer must verify that
/// the encoding is accurate.
pub unsafe trait Encode : PhantomFn<Self> {
    /// Returns the Objective-C type encoding for Self.
    fn encode() -> Encoding;
}

macro_rules! encode_impls {
    ($($t:ty : $s:expr,)*) => ($(
        unsafe impl Encode for $t {
            fn encode() -> Encoding { Encoding::from_str($s) }
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

unsafe impl Encode for isize {
    #[cfg(target_pointer_width = "32")]
    fn encode() -> Encoding { i32::encode() }

    #[cfg(target_pointer_width = "64")]
    fn encode() -> Encoding { i64::encode() }
}

unsafe impl Encode for usize {
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
        unsafe impl<'a $(, $t)*> $crate::Encode for &'a $name<$($t),*> {
            fn encode() -> Encoding { Encoding::from_str($code) }
        }

        unsafe impl<'a $(, $t)*> $crate::Encode for &'a mut $name<$($t),*> {
            fn encode() -> Encoding { Encoding::from_str($code) }
        }

        unsafe impl<'a $(, $t)*> $crate::Encode for Option<&'a $name<$($t),*>> {
            fn encode() -> Encoding { Encoding::from_str($code) }
        }

        unsafe impl<'a $(, $t)*> $crate::Encode for Option<&'a mut $name<$($t),*>> {
            fn encode() -> Encoding { Encoding::from_str($code) }
        }

        unsafe impl<$($t),*> $crate::Encode for *const $name<$($t),*> {
            fn encode() -> Encoding { Encoding::from_str($code) }
        }

        unsafe impl<$($t),*> $crate::Encode for *mut $name<$($t),*> {
            fn encode() -> Encoding { Encoding::from_str($code) }
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
        assert!(u32::encode() == encode!(u32));
        assert!(<()>::encode() == "v");
        assert!(<&Object>::encode() == "@");
        assert!(<*mut Object>::encode() == "@");
        assert!(<&Class>::encode() == "#");
        assert!(Sel::encode() == encode!(Sel));
    }
}

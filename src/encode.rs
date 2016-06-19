use std::ffi::CStr;
use std::fmt;
use std::os::raw::{c_char, c_void};
use std::str;
use malloc_buf::MallocBuffer;

use runtime::{Class, Sel};
use Message;

const QUALIFIERS: &'static [char] = &[
    'r', // const
    'n', // in
    'N', // inout
    'o', // out
    'O', // bycopy
    'R', // byref
    'V', // oneway
];

#[cfg(target_pointer_width = "64")]
const CODE_INLINE_CAP: usize = 30;

#[cfg(target_pointer_width = "32")]
const CODE_INLINE_CAP: usize = 14;

enum Code {
    Slice(&'static str),
    Owned(String),
    Inline(u8, [u8; CODE_INLINE_CAP]),
    Malloc(MallocBuffer<u8>)
}

/// An Objective-C type encoding.
///
/// For more information, see Apple's documentation:
/// https://developer.apple.com/library/mac/documentation/Cocoa/Conceptual/ObjCRuntimeGuide/Articles/ocrtTypeEncodings.html
pub struct Encoding {
    code: Code,
}

impl Encoding {
    /// Constructs an `Encoding` from its string representation.
    /// Unsafe because the caller must ensure the string is a valid encoding.
    pub unsafe fn from_str(code: &str) -> Encoding {
        from_str(code)
    }

    /// Returns self as a `str`.
    pub fn as_str(&self) -> &str {
        match self.code {
            Code::Slice(code) => code,
            Code::Owned(ref code) => code,
            Code::Inline(len, ref bytes) => unsafe {
                str::from_utf8_unchecked(&bytes[..len as usize])
            },
            Code::Malloc(ref buf) => unsafe {
                str::from_utf8_unchecked(&buf[..buf.len() - 1])
            },
        }
    }

    fn prepend(self, pre: &str) -> Encoding {
        let old_len = self.as_str().len();
        let len = pre.len() + old_len;
        if len <= CODE_INLINE_CAP {
            let mut bytes = [0; CODE_INLINE_CAP];
            (&mut bytes[..pre.len()]).copy_from_slice(pre.as_bytes());
            (&mut bytes[pre.len()..len]).copy_from_slice(self.as_str().as_bytes());
            Encoding { code: Code::Inline(len as u8, bytes) }
        } else if let Code::Owned(mut code) = self.code {
            unsafe {
                let bytes = code.as_mut_vec();
                // Make room for the prefix
                bytes.resize(len, 0);
                // Shift the old string left by the prefix length
                let old = bytes.as_ptr();
                let new = bytes.as_mut_ptr().offset(pre.len() as isize);
                ::std::ptr::copy(old, new, old_len);
                // Finally copy the prefix into the beginning
                (&mut bytes[..pre.len()]).copy_from_slice(pre.as_bytes());
            }
            Encoding { code: Code::Owned(code) }
        } else {
            let mut code = pre.to_owned();
            code.push_str(self.as_str());
            Encoding { code: Code::Owned(code) }
        }
    }
}

impl Clone for Encoding {
    fn clone(&self) -> Encoding {
        if let Code::Slice(code) = self.code {
            from_static_str(code)
        } else {
            from_str(self.as_str())
        }
    }
}

impl PartialEq for Encoding {
    fn eq(&self, other: &Encoding) -> bool {
        // strip qualifiers when comparing
        let s = self.as_str().trim_left_matches(QUALIFIERS);
        let o = other.as_str().trim_left_matches(QUALIFIERS);
        s == o
    }
}

impl fmt::Debug for Encoding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub fn from_static_str(code: &'static str) -> Encoding {
    Encoding { code: Code::Slice(code) }
}

pub fn from_str(code: &str) -> Encoding {
    if code.len() > CODE_INLINE_CAP {
        Encoding { code: Code::Owned(code.to_owned()) }
    } else {
        let mut bytes = [0; CODE_INLINE_CAP];
        (&mut bytes[..code.len()]).copy_from_slice(code.as_bytes());
        Encoding { code: Code::Inline(code.len() as u8, bytes) }
    }
}

pub unsafe fn from_malloc_str(ptr: *mut c_char) -> Encoding {
    let s = CStr::from_ptr(ptr);
    let bytes = s.to_bytes_with_nul();
    assert!(str::from_utf8(bytes).is_ok());
    let buf = MallocBuffer::new(ptr as *mut u8, bytes.len()).unwrap();
    Encoding { code: Code::Malloc(buf) }
}

/// Types that have an Objective-C type encoding.
///
/// Unsafe because Objective-C will make assumptions about the type (like its
/// size and alignment) from its encoding, so the implementer must verify that
/// the encoding is accurate.
pub unsafe trait Encode {
    /// Returns the Objective-C type encoding for Self.
    fn encode() -> Encoding;
}

macro_rules! encode_impls {
    ($($t:ty : $s:expr,)*) => ($(
        unsafe impl Encode for $t {
            fn encode() -> Encoding { from_static_str($s) }
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
    *mut Class: "#",
    *const Class: "#",
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

unsafe impl<T> Encode for *mut T where T: Message {
    default fn encode() -> Encoding {
        from_static_str("@")
    }
}

unsafe impl<T> Encode for *const T where T: Message {
    default fn encode() -> Encoding {
        from_static_str("@")
    }
}

unsafe impl<T> Encode for *mut T where T: Encode {
    default fn encode() -> Encoding {
        T::encode().prepend("^")
    }
}

unsafe impl<T> Encode for *const T where *mut T: Encode {
    default fn encode() -> Encoding {
        <*mut T>::encode().prepend("r")
    }
}

unsafe impl<'a, T> Encode for &'a T where *const T: Encode {
    default fn encode() -> Encoding {
        <*const T>::encode()
    }
}

unsafe impl<'a, T> Encode for &'a mut T where *mut T: Encode {
    default fn encode() -> Encoding {
        <*mut T>::encode()
    }
}

unsafe impl<'a , T> Encode for Option<&'a T> where *const T: Encode {
    default fn encode() -> Encoding {
        <*const T>::encode()
    }
}

unsafe impl<'a , T> Encode for Option<&'a mut T> where *mut T: Encode {
    default fn encode() -> Encoding {
        <*mut T>::encode()
    }
}

/// Types that represent a group of arguments, where each has an Objective-C
/// type encoding.
pub trait EncodeArguments {
    /// The type as which the encodings for Self will be returned.
    type Encs: AsRef<[Encoding]>;

    /// Returns the Objective-C type encodings for Self.
    fn encodings() -> Self::Encs;
}

macro_rules! count_idents {
    () => (0);
    ($a:ident) => (1);
    ($a:ident, $($b:ident),+) => (1 + count_idents!($($b),*));
}

macro_rules! encode_args_impl {
    ($($t:ident),*) => (
        impl<$($t: Encode),*> EncodeArguments for ($($t,)*) {
            type Encs = [Encoding; count_idents!($($t),*)];

            fn encodings() -> Self::Encs {
                [
                    $($t::encode()),*
                ]
            }
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
pub fn maybe_encode<T>() -> Option<Encoding> {
    trait MaybeEncode {
        fn maybe_encode() -> Option<Encoding>;
    }

    impl<T> MaybeEncode for T {
        default fn maybe_encode() -> Option<Encoding> {
            None
        }
    }

    impl<T: Encode> MaybeEncode for T {
        fn maybe_encode() -> Option<Encoding> {
            Some(T::encode())
        }
    }

    T::maybe_encode()
}

#[cfg(test)]
mod tests {
    use runtime::{Class, Object, Sel};
    use super::{Encode, Encoding};

    #[test]
    fn test_encode() {
        assert!(u32::encode().as_str() == "I");
        assert!(<()>::encode().as_str() == "v");
        assert!(<&Object>::encode().as_str() == "@");
        assert!(<*mut Object>::encode().as_str() == "@");
        assert!(<&Class>::encode().as_str() == "#");
        assert!(Sel::encode().as_str() == ":");
        assert!(<*mut u32>::encode().as_str() == "^I");
        assert!(<*mut ::std::os::raw::c_char>::encode().as_str() == "*");
    }

    #[test]
    fn test_inline_encoding() {
        let enc = unsafe { Encoding::from_str("C") };
        assert!(enc.as_str() == "C");

        let enc2 = enc.clone();
        assert!(enc2 == enc);
        assert!(enc2.as_str() == "C");
    }

    #[test]
    fn test_owned_encoding() {
        let s = "{Test=CCCCCCCCCCCCCCCCCCCCCCCCC}";
        let enc = unsafe { Encoding::from_str(s) };
        assert!(enc.as_str() == s);

        let enc2 = enc.clone();
        assert!(enc2 == enc);
        assert!(enc2.as_str() == s);
    }
}

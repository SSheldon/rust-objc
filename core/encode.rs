use runtime::{Class, Object, Sel};

/// A simple wrapper around a static string slice to contain the Objective-C
/// type encoding for a type. This is necessary for the `Encode` trait.
pub struct Encoding<T>(pub &'static str);

/// Types that have an Objective-C type encoding. For more information, see
/// Apple's documentation:
/// https://developer.apple.com/library/mac/documentation/Cocoa/Conceptual/ObjCRuntimeGuide/Articles/ocrtTypeEncodings.html
pub trait Encode {
    /// Return the Encoding for Self.
    fn code() -> Encoding<Self>;
}

impl Encode for i8 {
    fn code() -> Encoding<i8> { Encoding("c") }
}

impl Encode for i16 {
    fn code() -> Encoding<i16> { Encoding("s") }
}

impl Encode for i32 {
    fn code() -> Encoding<i32> { Encoding("i") }
}

impl Encode for i64 {
    fn code() -> Encoding<i64> { Encoding("q") }
}

impl Encode for isize {
    #[cfg(target_pointer_width = "32")]
    fn code() -> Encoding<isize> { Encoding("i") }

    #[cfg(target_pointer_width = "64")]
    fn code() -> Encoding<isize> { Encoding("q") }
}

impl Encode for u8 {
    fn code() -> Encoding<u8> { Encoding("C") }
}

impl Encode for u16 {
    fn code() -> Encoding<u16> { Encoding("S") }
}

impl Encode for u32 {
    fn code() -> Encoding<u32> { Encoding("I") }
}

impl Encode for u64 {
    fn code() -> Encoding<u64> { Encoding("Q") }
}

impl Encode for usize {
    #[cfg(target_pointer_width = "32")]
    fn code() -> Encoding<usize> { Encoding("I") }

    #[cfg(target_pointer_width = "64")]
    fn code() -> Encoding<usize> { Encoding("Q") }
}

impl Encode for f32 {
    fn code() -> Encoding<f32> { Encoding("f") }
}

impl Encode for f64 {
    fn code() -> Encoding<f64> { Encoding("d") }
}

impl Encode for bool {
    fn code() -> Encoding<bool> { Encoding("B") }
}

impl Encode for () {
    fn code() -> Encoding<()> { Encoding("v") }
}

impl Encode for Sel {
    fn code() -> Encoding<Sel> { Encoding(":") }
}

encode_message_impl!("@", Object);

encode_message_impl!("#", Class);

/// Returns the Objective-C type encoding for a type.
pub fn encode<T>() -> &'static str where T: Encode {
    let Encoding(code): Encoding<T> = Encode::code();
    code
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

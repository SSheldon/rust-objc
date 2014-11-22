use runtime::{Class, Object, Sel};

pub struct Encoding<T>(pub &'static str);

pub trait Encode {
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

impl Encode for int {
	#[cfg(target_word_size = "32")]
	fn code() -> Encoding<int> { Encoding("i") }

	#[cfg(target_word_size = "64")]
	fn code() -> Encoding<int> { Encoding("q") }
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

impl Encode for uint {
	#[cfg(target_word_size = "32")]
	fn code() -> Encoding<uint> { Encoding("I") }

	#[cfg(target_word_size = "64")]
	fn code() -> Encoding<uint> { Encoding("Q") }
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

encode_message_impl!("@", Object)

encode_message_impl!("#", Class)

impl Encode for Sel {
	fn code() -> Encoding<Sel> { Encoding(":") }
}

pub fn encode<T: Encode>() -> &'static str {
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

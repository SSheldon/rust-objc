use std::mem;

use runtime::{Class, Message, Sel};

pub struct Encoding<T>(&'static str);

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
	fn code() -> Encoding<int> {
		match mem::size_of::<int>() {
			1 => Encoding("c"),
			2 => Encoding("s"),
			4 => Encoding("i"),
			8 => Encoding("q"),
			_ => Encoding("?"),
		}
	}
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
	fn code() -> Encoding<uint> {
		match mem::size_of::<uint>() {
			1 => Encoding("C"),
			2 => Encoding("S"),
			4 => Encoding("I"),
			8 => Encoding("Q"),
			_ => Encoding("?"),
		}
	}
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

impl<'a, T: Message> Encode for &'a T {
	fn code() -> Encoding<&'a T> { Encoding("@") }
}

impl<'a, T: Message> Encode for &'a mut T {
	fn code() -> Encoding<&'a mut T> { Encoding("@") }
}

impl<T: Message> Encode for *const T {
	fn code() -> Encoding<*const T> { Encoding("@") }
}

impl<T: Message> Encode for *mut T {
	fn code() -> Encoding<*mut T> { Encoding("@") }
}

impl Encode for Class {
	fn code() -> Encoding<Class> { Encoding("#") }
}

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
		assert!(encode::<Class>() == "#");
		assert!(encode::<Sel>() == ":");
	}
}

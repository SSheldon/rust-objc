use runtime::Class;
use foundation::INSObject;

pub struct ClassName<T> {
	name: &'static str,
}

impl<T> ClassName<T> {
	pub fn from_str(name: &'static str) -> ClassName<T> {
		ClassName { name: name }
	}

	pub fn as_str(&self) -> &'static str {
		self.name
	}
}

pub fn class<T: INSObject>() -> Class {
	let name: ClassName<T> = INSObject::class_name();
	match Class::get(name.as_str()) {
		Some(cls) => cls,
		None => fail!("Class {} not found", name.as_str()),
	}
}

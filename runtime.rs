use std::str::raw::c_str_to_static_slice;

enum Selector { }
pub enum Object { }

pub trait Messageable {
	fn as_ptr(&self) -> *Object;

	fn is_nil(&self) -> bool {
		self.as_ptr().is_null()
	}
}

pub struct Sel {
	ptr: *Selector,
}

pub struct Class {
	ptr: *Object,
}

#[link(name = "Foundation", kind = "framework")]
extern {
	pub fn sel_registerName(name: *i8) -> Sel;
	pub fn sel_getName(sel: Sel) -> *i8;

	pub fn objc_getClass(name: *i8) -> Class;
	pub fn class_getName(cls: Class) -> *i8;

	pub fn objc_msgSend(obj: *Object, op: Sel, ...) -> *Object;
}

impl Sel {
	pub fn register(name: &str) -> Sel {
		name.with_c_str(|name| unsafe {
			sel_registerName(name)
		})
	}

	pub fn name(&self) -> &str {
		unsafe {
			let name = sel_getName(*self);
			c_str_to_static_slice(name)
		}
	}
}

impl PartialEq for Sel {
	fn eq(&self, other: &Sel) -> bool {
		self.ptr == other.ptr
	}
}

impl Eq for Sel { }

impl Clone for Sel {
	fn clone(&self) -> Sel {
		Sel { ptr: self.ptr }
	}
}

impl Messageable for Object {
	fn as_ptr(&self) -> *Object {
		self as *Object
	}
}

impl Messageable for *Object {
	fn as_ptr(&self) -> *Object {
		*self
	}
}

impl Class {
	pub fn get(name: &str) -> Class {
		name.with_c_str(|name| unsafe {
			objc_getClass(name)
		})
	}

	pub fn name(&self) -> &str {
		unsafe {
			let name = class_getName(*self);
			c_str_to_static_slice(name)
		}
	}
}

impl Messageable for Class {
	fn as_ptr(&self) -> *Object {
		self.ptr
	}
}

use std::mem;

use runtime::{Class, Messageable, Object, Sel, objc_msgSend};
use id::{Id, FromId};

pub trait INSObject : Messageable {
	unsafe fn retain(&self) {
		let retain = Sel::register("retain");
		objc_msgSend(self.as_ptr(), retain);
	}

	unsafe fn release(&self) {
		let release = Sel::register("release");
		objc_msgSend(self.as_ptr(), release);
	}

	fn hash_code(&self) -> uint {
		let hash = Sel::register("hash");
		unsafe {
			let result = objc_msgSend(self.as_ptr(), hash);
			mem::transmute(result)
		}
	}

	fn is_equal<T: INSObject>(&self, other: &T) -> bool {
		let is_equal = Sel::register("isEqual:");
		let result = unsafe {
			objc_msgSend(self.as_ptr(), is_equal, other.as_ptr())
		};
		!result.is_null()
	}
}

#[deriving(Clone, PartialEq, Eq, Hash)]
pub struct NSObject {
	ptr: Id,
}

impl Messageable for NSObject {
	unsafe fn as_ptr(&self) -> *Object {
		self.ptr.as_ptr()
	}
}

impl FromId for NSObject {
	unsafe fn from_id(id: Id) -> NSObject {
		NSObject { ptr: id }
	}
}

impl INSObject for NSObject { }

impl NSObject {
	fn class() -> Class {
		Class::get("NSObject")
	}

	pub fn new() -> NSObject {
		let class = NSObject::class();
		let alloc = Sel::register("alloc");
		let init = Sel::register("init");
		unsafe {
			let obj = objc_msgSend(class.as_ptr(), alloc);
			let obj = objc_msgSend(obj, init);
			FromId::from_retained_ptr(obj)
		}
	}
}

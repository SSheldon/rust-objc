use std::str::raw::c_str_to_static_slice;
use libc::{c_char, c_void, ptrdiff_t, size_t};

pub enum Object { }

pub struct Sel {
	ptr: *const c_void,
}

pub struct Ivar {
	_ptr: *const c_void,
}

pub struct Class {
	ptr: *mut Object,
}

pub type Imp = extern fn(*mut Object, Sel, ...) -> *mut Object;

#[link(name = "Foundation", kind = "framework")]
extern {
	pub fn sel_registerName(name: *const c_char) -> Sel;
	pub fn sel_getName(sel: Sel) -> *const c_char;

	pub fn objc_getClass(name: *const c_char) -> Class;
	pub fn class_getName(cls: Class) -> *const c_char;
	pub fn class_addMethod(cls: Class, name: Sel, imp: Imp, types: *const c_char) -> bool;
	pub fn class_addIvar(cls: Class, name: *const c_char, size: size_t, alignment: u8, types: *const c_char) -> bool;

	pub fn objc_allocateClassPair(superclass: Class, name: *const c_char, extraBytes: size_t) -> Class;
	pub fn objc_disposeClassPair(cls: Class);
	pub fn objc_registerClassPair(cls: Class);

	pub fn object_setInstanceVariable(obj: *mut Object, name: *const c_char, value: *mut c_void) -> Ivar;
	pub fn object_getInstanceVariable(obj: *mut Object, name: *const c_char, outValue: *mut *mut c_void) -> Ivar;
	pub fn object_setIvar(obj: *mut Object, ivar: Ivar, value: *mut Object);
	pub fn object_getIvar(obj: *mut Object, ivar: Ivar) -> *mut Object;
	pub fn object_getClass(obj: *mut Object) -> Class;

	pub fn ivar_getName(ivar: Ivar) -> *const c_char;
	pub fn ivar_getOffset(ivar: Ivar) -> ptrdiff_t;
	pub fn ivar_getTypeEncoding(ivar: Ivar) -> *const c_char;

	pub fn objc_msgSend(obj: *mut Object, op: Sel, ...) -> *mut Object;
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

impl Ivar {
	pub fn name(&self) -> &str {
		unsafe {
			let name = ivar_getName(*self);
			c_str_to_static_slice(name)
		}
	}

	pub fn offset(&self) -> int {
		let offset = unsafe {
			ivar_getOffset(*self)
		};
		offset as int
	}

	pub fn type_encoding(&self) -> &str {
		unsafe {
			let encoding = ivar_getName(*self);
			c_str_to_static_slice(encoding)
		}
	}
}

impl Object {
	unsafe fn get_ivar_ptr<T>(obj: *mut Object, name: &str) -> *mut T {
		if obj.is_null() {
			RawPtr::null()
		} else {
			let ivar = name.with_c_str(|name| {
				object_getInstanceVariable(obj, name, RawPtr::null())
			});
			let bytes = obj as *mut u8;
			let offset = ivar.offset();
			bytes.offset(offset) as *mut T
		}
	}

	pub unsafe fn get_ivar<T>(&self, name: &str) -> &T {
		let obj = self as *const Object as *mut Object;
		&*Object::get_ivar_ptr(obj, name)
	}

	pub unsafe fn get_mut_ivar<T>(&mut self, name: &str) -> &mut T {
		&mut *Object::get_ivar_ptr(self, name)
	}

	pub unsafe fn set_ivar<T>(&mut self, name: &str, value: T) {
		*self.get_mut_ivar::<T>(name) = value;
	}
}

impl Class {
	pub fn get(name: &str) -> Option<Class> {
		let cls = name.with_c_str(|name| unsafe {
			objc_getClass(name)
		});
		if cls.is_nil() {
			None
		} else {
			Some(cls)
		}
	}

	pub fn name(&self) -> &str {
		unsafe {
			let name = class_getName(*self);
			c_str_to_static_slice(name)
		}
	}
}

pub trait Message { }

impl Message for Object { }

pub trait ToMessage {
	fn as_ptr(&self) -> *mut Object;

	fn is_nil(&self) -> bool {
		self.as_ptr().is_null()
	}
}

impl<T: Message> ToMessage for *mut T {
	fn as_ptr(&self) -> *mut Object {
		*self as *mut Object
	}
}

impl<'a, T: Message> ToMessage for &'a T {
	fn as_ptr(&self) -> *mut Object {
		(*self as *const T as *mut T).as_ptr()
	}
}

impl ToMessage for Class {
	fn as_ptr(&self) -> *mut Object {
		self.ptr
	}
}

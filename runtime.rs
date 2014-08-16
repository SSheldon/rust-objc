use std::kinds::marker::NoCopy;
use std::slice::raw::buf_as_slice;
use std::str::raw::c_str_to_static_slice;
use libc::{c_char, c_uint, c_void, ptrdiff_t, size_t};
use libc;

use {encode, Encode};

pub struct Sel {
	ptr: *const c_void,
}

pub struct Ivar {
	nocopy: NoCopy,
}

pub struct Method {
	nocopy: NoCopy,
}

pub struct Class {
	nocopy: NoCopy,
}

pub struct Object {
	nocopy: NoCopy,
}

pub type Imp = extern fn(*mut Object, Sel, ...) -> *mut Object;

#[link(name = "Foundation", kind = "framework")]
extern {
	pub fn sel_registerName(name: *const c_char) -> Sel;
	pub fn sel_getName(sel: Sel) -> *const c_char;

	pub fn objc_getClass(name: *const c_char) -> *const Class;
	pub fn class_getName(cls: *const Class) -> *const c_char;
	pub fn class_getInstanceSize(cls: *const Class) -> size_t;
	pub fn class_getInstanceMethod(cls: *const Class, sel: Sel) -> *const Method;
	pub fn class_getInstanceVariable(cls: *const Class, name: *const c_char) -> *const Ivar;
	pub fn class_copyIvarList(cls: *const Class, outCount: *mut c_uint) -> *mut *const Ivar;
	pub fn class_addMethod(cls: *mut Class, name: Sel, imp: Imp, types: *const c_char) -> bool;
	pub fn class_addIvar(cls: *mut Class, name: *const c_char, size: size_t, alignment: u8, types: *const c_char) -> bool;

	pub fn objc_allocateClassPair(superclass: *const Class, name: *const c_char, extraBytes: size_t) -> *mut Class;
	pub fn objc_disposeClassPair(cls: *mut Class);
	pub fn objc_registerClassPair(cls: *mut Class);

	pub fn object_getClass(obj: *const Object) -> *const Class;

	pub fn ivar_getName(ivar: *const Ivar) -> *const c_char;
	pub fn ivar_getOffset(ivar: *const Ivar) -> ptrdiff_t;
	pub fn ivar_getTypeEncoding(ivar: *const Ivar) -> *const c_char;

	pub fn objc_msgSend(obj: *mut Object, op: Sel, ...) -> *mut Object;

	pub fn method_getName(method: *const Method) -> Sel;
	pub fn method_getImplementation(method: *const Method) -> Imp;
	pub fn method_getTypeEncoding(method: *const Method) -> *const c_char;
	pub fn method_getNumberOfArguments(method: *const Method) -> c_uint;
	pub fn method_setImplementation(method: *mut Method, imp: Imp) -> Imp;
	pub fn method_exchangeImplementations(m1: *mut Method, m2: *mut Method);
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
	fn clone(&self) -> Sel { *self }
}

impl Ivar {
	pub fn name(&self) -> &str {
		unsafe {
			let name = ivar_getName(self);
			c_str_to_static_slice(name)
		}
	}

	pub fn offset(&self) -> int {
		let offset = unsafe {
			ivar_getOffset(self)
		};
		offset as int
	}

	pub fn type_encoding(&self) -> &str {
		unsafe {
			let encoding = ivar_getTypeEncoding(self);
			c_str_to_static_slice(encoding)
		}
	}
}

impl Method {
	pub fn name(&self) -> Sel {
		unsafe {
			method_getName(self)
		}
	}

	pub fn type_encoding(&self) -> &str {
		unsafe {
			let encoding = method_getTypeEncoding(self);
			c_str_to_static_slice(encoding)
		}
	}

	pub fn arguments(&self) -> uint {
		unsafe {
			method_getNumberOfArguments(self) as uint
		}
	}

	pub fn implementation(&self) -> Imp {
		unsafe {
			method_getImplementation(self)
		}
	}

	pub unsafe fn set_implementation(&mut self, imp: Imp) -> Imp {
		method_setImplementation(self, imp)
	}

	pub unsafe fn exchange_implementation(&mut self, other: &mut Method) {
		method_exchangeImplementations(self, other);
	}
}

impl Class {
	pub fn get(name: &str) -> Option<&'static Class> {
		let cls = name.with_c_str(|name| unsafe {
			objc_getClass(name)
		});
		if cls.is_null() {
			None
		} else {
			Some(unsafe { &*cls })
		}
	}

	pub fn name(&self) -> &str {
		unsafe {
			let name = class_getName(self);
			c_str_to_static_slice(name)
		}
	}

	pub fn instance_size(&self) -> uint {
		unsafe {
			class_getInstanceSize(self) as uint
		}
	}

	pub fn instance_method(&self, sel: Sel) -> Option<&Method> {
		let method = unsafe {
			class_getInstanceMethod(self, sel)
		};
		if method.is_null() {
			None
		} else {
			Some(unsafe { &*method })
		}
	}

	pub fn instance_variable(&self, name: &str) -> Option<&Ivar> {
		let ivar = name.with_c_str(|name| unsafe {
			class_getInstanceVariable(self, name)
		});
		if ivar.is_null() {
			None
		} else {
			Some(unsafe { &*ivar })
		}
	}

	pub fn instance_variables(&self) -> Vec<&Ivar> {
		unsafe {
			let mut count: c_uint = 0;
			let ivars = class_copyIvarList(self, &mut count);
			let vec = buf_as_slice(ivars as *const _, count as uint, |ivars| {
				ivars.to_vec()
			});
			libc::free(ivars as *mut c_void);
			vec
		}
	}
}

impl Object {
	pub fn class(&self) -> &'static Class {
		unsafe {
			&*object_getClass(self)
		}
	}

	pub unsafe fn get_ivar<T: Encode>(&self, name: &str) -> &T {
		let cls = self.class();
		let ptr = match cls.instance_variable(name) {
			Some(ivar) => {
				assert!(ivar.type_encoding() == encode::<T>());
				let offset = ivar.offset();
				let self_ptr = self as *const Object;
				(self_ptr as *const u8).offset(offset) as *const T
			}
			None => fail!("Ivar {} not found on class {}", name, cls.name()),
		};
		&*ptr
	}

	pub unsafe fn get_mut_ivar<T: Encode>(&mut self, name: &str) -> &mut T {
		let cls = self.class();
		let ptr = match cls.instance_variable(name) {
			Some(ivar) => {
				assert!(ivar.type_encoding() == encode::<T>());
				let offset = ivar.offset();
				let self_ptr = self as *mut Object;
				(self_ptr as *mut u8).offset(offset) as *mut T
			}
			None => fail!("Ivar {} not found on class {}", name, cls.name()),
		};
		&mut *ptr
	}

	pub unsafe fn set_ivar<T: Encode>(&mut self, name: &str, value: T) {
		*self.get_mut_ivar::<T>(name) = value;
	}
}

pub trait Message { }

impl Message for Object { }

impl Message for Class { }

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

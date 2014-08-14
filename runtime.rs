use std::slice::raw::buf_as_slice;
use std::str::raw::c_str_to_static_slice;
use libc::{c_char, c_uint, c_void, ptrdiff_t, size_t};
use libc;

use {encode, Encode};

pub enum Object { }

pub struct Sel {
	ptr: *const c_void,
}

pub struct Ivar {
	ptr: *const c_void,
}

pub struct Method {
	ptr: *mut c_void,
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
	pub fn class_getInstanceSize(cls: Class) -> size_t;
	pub fn class_getInstanceMethod(cls: Class, sel: Sel) -> Method;
	pub fn class_getInstanceVariable(cls: Class, name: *const c_char) -> Ivar;
	pub fn class_copyIvarList(cls: Class, outCount: *mut c_uint) -> *mut Ivar;
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

	pub fn method_getName(method: Method) -> Sel;
	pub fn method_getImplementation(method: Method) -> Imp;
	pub fn method_getTypeEncoding(method: Method) -> *const c_char;
	pub fn method_getNumberOfArguments(method: Method) -> c_uint;
	pub fn method_setImplementation(method: Method, imp: Imp) -> Imp;
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
			let encoding = ivar_getTypeEncoding(*self);
			c_str_to_static_slice(encoding)
		}
	}
}

impl Clone for Ivar {
	fn clone(&self) -> Ivar { *self }
}

impl Method {
	pub fn name(&self) -> Sel {
		unsafe {
			method_getName(*self)
		}
	}

	pub fn type_encoding(&self) -> &str {
		unsafe {
			let encoding = method_getTypeEncoding(*self);
			c_str_to_static_slice(encoding)
		}
	}

	pub fn arguments(&self) -> uint {
		unsafe {
			method_getNumberOfArguments(*self) as uint
		}
	}

	pub fn implementation(&self) -> Imp {
		unsafe {
			method_getImplementation(*self)
		}
	}

	pub unsafe fn set_implementation(&mut self, imp: Imp) -> Imp {
		method_setImplementation(*self, imp)
	}
}

impl Object {
	pub fn class(&self) -> Class {
		unsafe {
			object_getClass(self as *const Object as *mut Object)
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

impl Class {
	pub fn get(name: &str) -> Option<Class> {
		let cls = name.with_c_str(|name| unsafe {
			objc_getClass(name)
		});
		if cls.ptr.is_null() {
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

	pub fn instance_size(&self) -> uint {
		unsafe {
			class_getInstanceSize(*self) as uint
		}
	}

	pub fn instance_method(&self, sel: Sel) -> Option<Method> {
		let method = unsafe {
			class_getInstanceMethod(*self, sel)
		};
		if method.ptr.is_null() {
			None
		} else {
			Some(method)
		}
	}

	pub fn instance_variable(&self, name: &str) -> Option<Ivar> {
		let ivar = name.with_c_str(|name| unsafe {
			class_getInstanceVariable(*self, name)
		});
		if ivar.ptr.is_null() {
			None
		} else {
			Some(ivar)
		}
	}

	pub fn instance_variables(&self) -> Vec<Ivar> {
		unsafe {
			let mut count: c_uint = 0;
			let ivars = class_copyIvarList(*self, &mut count) as *const Ivar;
			let vec = buf_as_slice(ivars, count as uint, |ivars| {
				ivars.to_vec()
			});
			libc::free(ivars as *mut c_void);
			vec
		}
	}
}

impl Clone for Class {
	fn clone(&self) -> Class { *self }
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

use std::mem;
use libc::size_t;

use {encode, Encode};
use runtime::{Class, Imp, Sel};
use runtime;

pub struct MethodDecl {
	pub sel: Sel,
	pub imp: Imp,
	pub types: String,
}

pub struct ClassDecl {
	cls: *mut Class,
}

impl ClassDecl {
	pub fn new(superclass: &Class, name: &str) -> Option<ClassDecl> {
		let cls = name.with_c_str(|name| unsafe {
			runtime::objc_allocateClassPair(superclass, name, 0)
		});
		if cls.is_null() {
			None
		} else {
			Some(ClassDecl { cls: cls })
		}
	}

	pub fn add_method(&mut self, method: MethodDecl) -> bool {
		method.types.with_c_str(|types| unsafe {
			runtime::class_addMethod(self.cls, method.sel, method.imp, types)
		})
	}

	pub fn add_ivar<T: Encode>(&mut self, name: &str) -> bool {
		let types = encode::<T>();
		let size = mem::size_of::<T>() as size_t;
		let align = mem::align_of::<T>() as u8;
		types.with_c_str(|types| {
			name.with_c_str(|name| unsafe {
				runtime::class_addIvar(self.cls, name, size, align, types)
			})
		})
	}

	pub fn register(self) -> &'static Class {
		unsafe {
			let cls = self.cls;
			runtime::objc_registerClassPair(cls);
			// Forget self otherwise the class will be disposed in drop
			mem::forget(self);
			&*cls
		}
	}
}

impl Drop for ClassDecl {
	fn drop(&mut self) {
		unsafe {
			runtime::objc_disposeClassPair(self.cls);
		}
	}
}

#[cfg(test)]
mod tests {
	use runtime::{Class, Object};
	use super::ClassDecl;

	#[test]
	fn test_custom_class() {
		let superclass = Class::get("NSObject").unwrap();
		let decl = ClassDecl::new(superclass, "MyObject");
		assert!(decl.is_some());
		let mut decl = decl.unwrap();

		decl.add_ivar::<uint>("_foo");
		decl.add_method(method!(
			(*mut Object)_this, doNothing; {
				()
			}
		));
		decl.add_method(method!(
			(&mut Object)this, setFoo:(uint)foo; {
				unsafe {
					this.set_ivar::<uint>("_foo", foo);
				}
			}
		));
		decl.add_method(method!(
			(&Object)this, foo -> uint {
				unsafe {
					*this.get_ivar::<uint>("_foo")
				}
			}
		));
		decl.add_method(method!(
			(*mut Object)this, doSomethingWithFoo:(uint)_foo -> *mut Object {
				this
			}
		));

		let cls = decl.register();
		unsafe {
			let obj = msg_send![cls alloc];
			let obj = msg_send![obj init];

			msg_send![obj doNothing];
			msg_send![obj setFoo:13u];
			let result = msg_send![obj foo] as uint;
			assert!(result == 13);
			let result = msg_send![obj doSomethingWithFoo:13u];
			assert!(result == obj);

			msg_send![obj release];
		}
	}
}

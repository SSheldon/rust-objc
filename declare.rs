use std::mem;

use runtime::{Class, Imp, Sel, ToMessage};
use runtime;

pub struct MethodDecl {
	pub sel: Sel,
	pub imp: Imp,
	pub types: String,
}

pub struct ClassDecl {
	cls: Class,
}

impl ClassDecl {
	pub fn new(superclass: &Class, name: &str) -> Option<ClassDecl> {
		let cls = name.with_c_str(|name| unsafe {
			runtime::objc_allocateClassPair(*superclass, name, 0)
		});
		if cls.is_nil() {
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

	pub fn register(self) -> Class {
		unsafe {
			runtime::objc_registerClassPair(self.cls);
		}
		let cls = self.cls;
		// Forget self otherwise the class will be disposed in drop
		unsafe {
			mem::forget(self);
		}
		cls
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
		let decl = ClassDecl::new(&superclass, "MyObject");
		assert!(decl.is_some());
		let mut decl = decl.unwrap();

		decl.add_method(method!(
			(*mut Object)_this, doNothing; {
				()
			}
		));
		decl.add_method(method!(
			(*mut Object)_this, doNothingWithFoo:(uint)_foo; {
				()
			}
		));
		decl.add_method(method!(
			(*mut Object)this, doSomething -> *mut Object {
				this
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
			msg_send![obj doNothingWithFoo:0u];
			let result = msg_send![obj doSomething];
			assert!(result == obj);
			let result = msg_send![obj doSomethingWithFoo:0u];
			assert!(result == obj);

			msg_send![obj release];
		}
	}
}

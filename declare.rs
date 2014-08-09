use runtime::{Class, Imp, Sel, ToMessage};
use runtime;

pub struct MethodDecl {
	sel: Sel,
	imp: Imp,
	types: String,
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
		self.cls
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

		let method_decl = method!(
			(*mut Object)this -(*mut Object)doSomething {
				this
			}
		);
		decl.add_method(method_decl);

		let cls = decl.register();
		unsafe {
			let obj = msg_send![cls alloc];
			let obj = msg_send![obj init];
			let result = msg_send![obj doSomething];
			assert!(result == obj);
			msg_send![obj release];
		}
	}
}

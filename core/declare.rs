use std::ffi::CString;
use std::mem;
use libc::size_t;

use {encode, Encode};
use runtime::{Class, Imp, Sel};
use runtime;

/// A type for declaring a new method.
/// `MethodDecl`s are usually created using the `method!` macro.
pub struct MethodDecl {
    /// The method's selector.
    pub sel: Sel,
    /// The method's implementation.
    pub imp: Imp,
    /// The types of the method's arguments.
    pub types: String,
}

/// A type for declaring a new class and adding new methods and ivars to it
/// before registering it.
pub struct ClassDecl {
    cls: *mut Class,
}

impl ClassDecl {
    /// Constructs a `ClassDecl` with the given superclass and name.
    /// Returns `None` if the class couldn't be allocated.
    pub fn new(superclass: &Class, name: &str) -> Option<ClassDecl> {
        let name = CString::from_slice(name.as_bytes());
        let cls = unsafe {
            runtime::objc_allocateClassPair(superclass, name.as_ptr(), 0)
        };
        if cls.is_null() {
            None
        } else {
            Some(ClassDecl { cls: cls })
        }
    }

    /// Adds a method declared with the given `MethodDecl` to self.
    /// Returns true if the method was sucessfully added.
    pub fn add_method(&mut self, method: MethodDecl) -> bool {
        let MethodDecl { sel, imp, types } = method;
        let types = CString::from_vec(types.into_bytes());
        unsafe {
            runtime::class_addMethod(self.cls, sel, imp, types.as_ptr())
        }
    }

    /// Adds an ivar with type `T` and the provided name to self.
    /// Returns true if the ivar was sucessfully added.
    pub fn add_ivar<T: Encode>(&mut self, name: &str) -> bool {
        let name = CString::from_slice(name.as_bytes());
        let types = CString::from_slice(encode::<T>().as_bytes());
        let size = mem::size_of::<T>() as size_t;
        let align = mem::align_of::<T>() as u8;
        unsafe {
            runtime::class_addIvar(self.cls, name.as_ptr(), size, align,
                types.as_ptr())
        }
    }

    /// Registers self, consuming it and returning a reference to the
    /// newly registered `Class`.
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
    use std::mem;
    use encode;
    use runtime::{Class, Object, Sel};
    use super::{ClassDecl, MethodDecl};

    #[test]
    fn test_custom_class() {
        let superclass = Class::get("NSObject").unwrap();
        let decl = ClassDecl::new(superclass, "MyObject");
        assert!(decl.is_some());
        let mut decl = decl.unwrap();

        decl.add_ivar::<uint>("_foo");

        extern fn my_object_get_foo(this: &Object, _cmd: Sel) -> uint {
            unsafe {
                *this.get_ivar::<uint>("_foo")
            }
        }
        decl.add_method(MethodDecl {
            sel: Sel::register("foo"),
            imp: unsafe { mem::transmute(my_object_get_foo) },
            types: String::from_str(encode::<uint>()) + "@:",
        });

        extern fn my_object_set_foo(this: &mut Object, _cmd: Sel, foo: uint) {
            unsafe {
                this.set_ivar::<uint>("_foo", foo);
            }
        }
        decl.add_method(MethodDecl {
            sel: Sel::register("setFoo:"),
            imp: unsafe { mem::transmute(my_object_set_foo) },
            types: String::from_str("v@:") + encode::<uint>(),
        });

        let cls = decl.register();
        unsafe {
            let obj = msg_send![cls, alloc];
            let obj = msg_send![obj, init];

            msg_send![obj, setFoo:13u];
            let result = msg_send![obj, foo] as uint;
            assert!(result == 13);

            msg_send![obj, release];
        }
    }
}

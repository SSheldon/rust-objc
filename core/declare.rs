/*!
Functionality for declaring Objective-C classes.

Classes can be declared using the `ClassDecl` struct. Instance variables and
methods can then be added before the class is ultimately registered.

# Example

The following example demonstrates declaring a class named `MyNumber` that has
one ivar, a `u32` named `_number` and a `number` method that returns it:

```
# #[macro_use] extern crate objc;
# use objc::declare::ClassDecl;
# use objc::runtime::{Class, Object, Sel};
# fn main() {
let superclass = Class::get("NSObject").unwrap();
let mut decl = ClassDecl::new(superclass, "MyNumber").unwrap();

// Add an instance variable
decl.add_ivar::<u32>("_number");

// Add an ObjC method for getting the number
extern fn my_number_get(this: &Object, _cmd: Sel) -> u32 {
    unsafe { *this.get_ivar("_number") }
}
decl.add_method(sel!(number),
    my_number_get as extern fn(&Object, Sel) -> u32);

decl.register();
# }
```
*/

use std::ffi::CString;
use std::fmt;
use std::mem;
use libc::size_t;

use {Encode, Message};
use runtime::{Class, Imp, NO, Object, Sel, self};

/// An error returned from `IntoMethodImp::into_imp` to indicate that a
/// selector and function accept unequal numbers of arguments.
#[derive(Clone, PartialEq, Debug)]
pub struct UnequalArgsError {
    sel_args: usize,
    fn_args: usize,
}

impl fmt::Display for UnequalArgsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Selector accepts {} arguments, but function accepts {}",
            self.sel_args, self.fn_args)
    }
}

/// Types that can be used as the implementation of an Objective-C method.
pub trait IntoMethodImp {
    /// The callee type of the method.
    type Callee: Message;
    /// The return type of the method.
    type Ret;

    /// Returns the method type encoding for Self.
    fn method_encoding() -> String;

    /// Consumes self to create a method implementation for the given selector.
    ///
    /// Returns an error if self and the selector do not accept the same number
    /// of arguments.
    fn into_imp(self, sel: Sel) -> Result<Imp, UnequalArgsError>;
}

macro_rules! count_idents {
    () => (0);
    ($a:ident) => (1);
    ($a:ident, $($b:ident),+) => (1 + count_idents!($($b),*));
}

macro_rules! method_decl_impl {
    (-$s:ident, $sp:ty, $($t:ident),*) => (
        impl<$s, R $(, $t)*> IntoMethodImp for extern fn($sp, Sel $(, $t)*) -> R
                where $s: Message, R: Encode $(, $t: Encode)* {
            type Callee = $s;
            type Ret = R;

            fn method_encoding() -> String {
                let types = [
                    R::encode(),
                    <*mut Object>::encode(),
                    Sel::encode(),
                    $($t::encode()),*
                ];
                types.iter().map(|s| s.as_str()).collect()
            }

            fn into_imp(self, sel: Sel) -> Result<Imp, UnequalArgsError> {
                // Add 2 to the arguments for self and _cmd
                let fn_args = 2 + count_idents!($($t),*);
                let sel_args = 2 + sel.name().chars().filter(|&c| c == ':').count();
                if sel_args == fn_args {
                    unsafe { Ok(mem::transmute(self)) }
                } else {
                    Err(UnequalArgsError { sel_args: sel_args, fn_args: fn_args })
                }
            }
        }
    );
    ($($t:ident),*) => (
        method_decl_impl!(-T, &T, $($t),*);
        method_decl_impl!(-T, &mut T, $($t),*);
    );
}

method_decl_impl!();
method_decl_impl!(A);
method_decl_impl!(A, B);
method_decl_impl!(A, B, C);
method_decl_impl!(A, B, C, D);
method_decl_impl!(A, B, C, D, E);
method_decl_impl!(A, B, C, D, E, F);
method_decl_impl!(A, B, C, D, E, F, G);
method_decl_impl!(A, B, C, D, E, F, G, H);
method_decl_impl!(A, B, C, D, E, F, G, H, I);
method_decl_impl!(A, B, C, D, E, F, G, H, I, J);
method_decl_impl!(A, B, C, D, E, F, G, H, I, J, K);
method_decl_impl!(A, B, C, D, E, F, G, H, I, J, K, L);

/// A type for declaring a new class and adding new methods and ivars to it
/// before registering it.
pub struct ClassDecl {
    cls: *mut Class,
}

impl ClassDecl {
    /// Constructs a `ClassDecl` with the given superclass and name.
    /// Returns `None` if the class couldn't be allocated.
    pub fn new(superclass: &Class, name: &str) -> Option<ClassDecl> {
        let name = CString::new(name).unwrap();
        let cls = unsafe {
            runtime::objc_allocateClassPair(superclass, name.as_ptr(), 0)
        };
        if cls.is_null() {
            None
        } else {
            Some(ClassDecl { cls: cls })
        }
    }

    /// Adds a method with the given name and implementation to self.
    /// Panics if the method wasn't sucessfully added
    /// or if the selector and function take different numbers of arguments.
    pub fn add_method<F>(&mut self, sel: Sel, func: F) where F: IntoMethodImp {
        let types = CString::new(F::method_encoding()).unwrap();
        let imp = match func.into_imp(sel) {
            Ok(imp) => imp,
            Err(err) => panic!("{}", err),
        };
        let success = unsafe {
            runtime::class_addMethod(self.cls, sel, imp, types.as_ptr())
        };
        assert!(success != NO, "Failed to add method {:?}", sel);
    }

    /// Adds a class method with the given name and implementation to self.
    /// Panics if the method wasn't sucessfully added
    /// or if the selector and function take different numbers of arguments.
    pub fn add_class_method<F>(&mut self, sel: Sel, func: F)
            where F: IntoMethodImp<Callee=Class> {
        let types = CString::new(F::method_encoding()).unwrap();
        let imp = match func.into_imp(sel) {
            Ok(imp) => imp,
            Err(err) => panic!("{}", err),
        };
        let success = unsafe {
            let cls_obj = self.cls as *const Object;
            let metaclass = runtime::object_getClass(cls_obj) as *mut Class;
            runtime::class_addMethod(metaclass, sel, imp, types.as_ptr())
        };
        assert!(success != NO, "Failed to add class method {:?}", sel);
    }

    /// Adds an ivar with type `T` and the provided name to self.
    /// Panics if the ivar wasn't successfully added.
    pub fn add_ivar<T>(&mut self, name: &str) where T: Encode {
        let c_name = CString::new(name).unwrap();
        let encoding = CString::new(T::encode().as_str()).unwrap();
        let size = mem::size_of::<T>() as size_t;
        let align = mem::align_of::<T>() as u8;
        let success = unsafe {
            runtime::class_addIvar(self.cls, c_name.as_ptr(), size, align,
                encoding.as_ptr())
        };
        assert!(success != NO, "Failed to add ivar {}", name);
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
    use runtime::{Object, Sel};
    use test_utils;
    use super::IntoMethodImp;

    #[test]
    fn test_custom_class() {
        // Registering the custom class is in test_utils
        let obj = test_utils::custom_object();
        unsafe {
            let _: () = msg_send![obj, setFoo:13u32];
            let result: u32 = msg_send![obj, foo];
            assert!(result == 13);
        }
    }

    #[test]
    fn test_class_method() {
        let cls = test_utils::custom_class();
        unsafe {
            let result: u32 = msg_send![cls, classFoo];
            assert!(result == 7);
        }
    }

    #[test]
    fn test_mismatched_args() {
        extern fn wrong_num_args_method(_obj: &Object, _cmd: Sel, _a: i32) { }

        let sel = sel!(doSomethingWithFoo:bar:);
        let f: extern fn(&Object, Sel, i32) = wrong_num_args_method;
        let imp = f.into_imp(sel);
        assert!(imp.is_err());
    }
}

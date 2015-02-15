use std::ffi::CString;
use std::mem;
use libc::size_t;

use {encode, Encode, EncodePtr, Message};
use runtime::{Class, Imp, Sel, self};

/// A type for declaring a new method.
/// `MethodDecl`s are usually created using the `method!` macro.
pub struct MethodDecl {
    sel: Sel,
    imp: Imp,
    types: String,
}

impl MethodDecl {
    pub fn new<F>(sel: Sel, func: F) -> Result<MethodDecl, ()>
            where F: IntoMethodDecl {
        func.into_method_decl(sel)
    }
}

pub trait IntoMethodDecl {
    fn into_method_decl(self, sel: Sel) -> Result<MethodDecl, ()>;
}

macro_rules! count_idents {
    () => (0);
    ($a:ident) => (1);
    ($a:ident, $($b:ident),+) => (1 + count_idents!($($b),*));
}

macro_rules! method_decl_impl {
    (-$s:ident, $sp:ty, $($t:ident),*) => (
        impl<$s, R $(, $t)*> IntoMethodDecl for extern fn($sp, Sel $(, $t)*) -> R
                where $s: Message + EncodePtr, R: Encode $(, $t: Encode)* {
            fn into_method_decl(self, sel: Sel) -> Result<MethodDecl, ()> {
                let num_args = count_idents!($($t),*);
                if sel.name().chars().filter(|&c| c == ':').count() == num_args {
                    let imp: Imp = unsafe { mem::transmute(self) };

                    let mut types = encode::<R>().to_string();
                    types.push_str(encode::<$sp>());
                    types.push_str(encode::<Sel>());
                    $(types.push_str(encode::<$t>());)*

                    Ok(MethodDecl { sel: sel, imp: imp, types: types })
                } else {
                    Err(())
                }
            }
        }
    );
    ($($t:ident),*) => (
        method_decl_impl!(-T, &T, $($t),*);
        method_decl_impl!(-T, &mut T, $($t),*);
        method_decl_impl!(-T, Option<&T>, $($t),*);
        method_decl_impl!(-T, Option<&mut T>, $($t),*);
        method_decl_impl!(-T, *const T, $($t),*);
        method_decl_impl!(-T, *mut T, $($t),*);
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
    pub fn add_ivar<T>(&mut self, name: &str) -> bool where T: Encode {
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
    use runtime::{Class, Object, Sel};
    use super::{ClassDecl, MethodDecl};

    #[test]
    fn test_custom_class() {
        let superclass = Class::get("NSObject").unwrap();
        let decl = ClassDecl::new(superclass, "MyObject");
        assert!(decl.is_some());
        let mut decl = decl.unwrap();

        decl.add_ivar::<u32>("_foo");
        decl.add_method(method!(
            (*mut Object)_this, doNothing; {
                ()
            }
        ));
        decl.add_method(method!(
            (&mut Object)this, setFoo:(u32)foo, bar:(u32)_bar; {
                unsafe {
                    this.set_ivar::<u32>("_foo", foo);
                }
            }
        ));
        decl.add_method(method!(
            (&Object)this, foo -> u32, {
                unsafe {
                    *this.get_ivar::<u32>("_foo")
                }
            }
        ));
        decl.add_method(method!(
            (*mut Object)this, doSomethingWithFoo:(u32)_foo, bar:(u32)_bar -> *mut Object, {
                this
            }
        ));

        let cls = decl.register();
        unsafe {
            let obj: *mut Object = msg_send![cls, alloc];
            let obj: *mut Object = msg_send![obj, init];

            let _: () = msg_send![obj, doNothing];
            let _: () = msg_send![obj, setFoo:13u32 bar:0u32];
            let result: u32 = msg_send![obj, foo];
            assert!(result == 13);
            let result: *mut Object = msg_send![obj, doSomethingWithFoo:13u32
                                                                    bar:0u32];
            assert!(result == obj);

            let _: () = msg_send![obj, release];
        }
    }

    #[test]
    fn test_mismatched_args() {
        extern fn wrong_num_args_method(_obj: &Object, _cmd: Sel, _a: i32) { }

        let sel = sel!(doSomethingWithFoo:bar:);
        let f: extern fn(&Object, Sel, i32) = wrong_num_args_method;
        let decl = MethodDecl::new(sel, f);
        assert!(decl.is_err());
    }
}

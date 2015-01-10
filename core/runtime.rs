//! A Rust interface for the functionality of the Objective-C runtime.
//!
//! For more information on foreign functions, see Apple's documentation:
//! https://developer.apple.com/library/mac/documentation/Cocoa/Reference/ObjCRuntimeRef/index.html

use std::ffi::{CString, self};
use std::mem;
use std::ptr;
use std::slice;
use std::str;
use libc::{c_char, c_int, c_uint, c_void, ptrdiff_t, size_t};
use libc;

use {encode, Encode};

/// A type that represents a `malloc`'d chunk of memory.
pub struct MallocBuffer<T> {
    ptr: *mut T,
    len: usize,
}

impl<T> MallocBuffer<T> {
    /// Constructs a new `MallocBuffer` for a `malloc`'d buffer
    /// with the given length at the given pointer.
    /// Returns `None` if the given pointer is null.
    /// When this `MallocBuffer` drops, the buffer will be `free`'d.
    ///
    /// Unsafe because there must be `len` contiguous, valid instances of `T`
    /// at `ptr`, and `T` must not be a type that implements `Drop`
    /// (because the `MallocBuffer` makes no attempt to drop its elements,
    /// just the buffer containing them).
    pub unsafe fn new(ptr: *mut T, len: usize) -> Option<MallocBuffer<T>> {
        if ptr.is_null() {
            None
        } else {
            Some(MallocBuffer { ptr: ptr, len: len })
        }
    }
}

#[unsafe_destructor]
impl<T> Drop for MallocBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.ptr as *mut c_void);
        }
    }
}

impl<T> AsSlice<T> for MallocBuffer<T> {
    fn as_slice(&self) -> &[T] {
        let const_ptr = self.ptr as *const T;
        unsafe {
            let s = slice::from_raw_buf(&const_ptr, self.len);
            mem::transmute(s)
        }
    }
}

/// A type that represents a `malloc`'d string.
pub struct MallocString {
    data: MallocBuffer<u8>,
}

impl MallocString {
    /// Constructs a new `MallocString` for a `malloc`'d C string buffer.
    /// Returns `None` if the given pointer is null or the C string isn't UTF8.
    /// When this `MallocString` drops, the buffer will be `free`'d.
    ///
    /// Unsafe because `ptr` must point to a valid, nul-terminated C string.
    pub unsafe fn new(ptr: *mut c_char) -> Option<MallocString> {
        if ptr.is_null() {
            None
        } else {
            let const_ptr = ptr as *const c_char;
            let bytes = ffi::c_str_to_bytes(&const_ptr);
            if str::from_utf8(bytes).is_ok() {
                let data = MallocBuffer {
                    ptr: ptr as *mut u8,
                    // len + 1 to account for the nul byte
                    len: bytes.len() + 1,
                };
                Some(MallocString { data: data })
            } else {
                None
            }
        }
    }
}

impl Str for MallocString {
    fn as_slice(&self) -> &str {
        let v = self.data.as_slice().slice_to(self.data.len - 1);
        unsafe { str::from_utf8_unchecked(v) }
    }
}

unsafe fn from_c_str<'a>(ptr: *const c_char) -> &'a str {
    let bytes = ffi::c_str_to_bytes(&ptr);
    let s = str::from_utf8(bytes).unwrap();
    mem::transmute(s)
}

/// A type that represents a method selector.
#[repr(C)]
pub struct Sel {
    ptr: *const c_void,
}

/// A type that represents an instance variable.
#[allow(missing_copy_implementations)]
pub enum Ivar { }

/// A type that represents a method in a class definition.
#[allow(missing_copy_implementations)]
pub enum Method { }

/// A type that represents an Objective-C class.
#[allow(missing_copy_implementations)]
pub enum Class { }

/// A type that represents an instance of a class.
#[allow(missing_copy_implementations)]
pub enum Object { }

/// A pointer to the start of a method implementation.
pub type Imp = extern fn(*mut Object, Sel, ...) -> *mut Object;

#[allow(improper_ctypes)]
#[link(name = "Foundation", kind = "framework")]
extern {
    pub fn sel_registerName(name: *const c_char) -> Sel;
    pub fn sel_getName(sel: Sel) -> *const c_char;

    pub fn class_getName(cls: *const Class) -> *const c_char;
    pub fn class_getInstanceSize(cls: *const Class) -> size_t;
    pub fn class_getInstanceMethod(cls: *const Class, sel: Sel) -> *const Method;
    pub fn class_getInstanceVariable(cls: *const Class, name: *const c_char) -> *const Ivar;
    pub fn class_copyMethodList(cls: *const Class, outCount: *mut c_uint) -> *mut *const Method;
    pub fn class_copyIvarList(cls: *const Class, outCount: *mut c_uint) -> *mut *const Ivar;
    pub fn class_addMethod(cls: *mut Class, name: Sel, imp: Imp, types: *const c_char) -> bool;
    pub fn class_addIvar(cls: *mut Class, name: *const c_char, size: size_t, alignment: u8, types: *const c_char) -> bool;

    pub fn objc_allocateClassPair(superclass: *const Class, name: *const c_char, extraBytes: size_t) -> *mut Class;
    pub fn objc_disposeClassPair(cls: *mut Class);
    pub fn objc_registerClassPair(cls: *mut Class);

    pub fn object_getClass(obj: *const Object) -> *const Class;

    pub fn objc_getClassList(buffer: *mut *const Class, bufferLen: c_int) -> c_int;
    pub fn objc_copyClassList(outCount: *mut c_uint) -> *mut *const Class;
    pub fn objc_getClass(name: *const c_char) -> *const Class;

    pub fn ivar_getName(ivar: *const Ivar) -> *const c_char;
    pub fn ivar_getOffset(ivar: *const Ivar) -> ptrdiff_t;
    pub fn ivar_getTypeEncoding(ivar: *const Ivar) -> *const c_char;

    pub fn objc_msgSend(obj: *mut Object, op: Sel, ...) -> *mut Object;

    pub fn method_getName(method: *const Method) -> Sel;
    pub fn method_getImplementation(method: *const Method) -> Imp;
    pub fn method_getTypeEncoding(method: *const Method) -> *const c_char;
    pub fn method_copyReturnType(method: *const Method) -> *mut c_char;
    pub fn method_copyArgumentType(method: *const Method, index: c_uint) -> *mut c_char;
    pub fn method_getNumberOfArguments(method: *const Method) -> c_uint;
    pub fn method_setImplementation(method: *mut Method, imp: Imp) -> Imp;
    pub fn method_exchangeImplementations(m1: *mut Method, m2: *mut Method);
}

impl Sel {
    /// Registers a method with the Objective-C runtime system,
    /// maps the method name to a selector, and returns the selector value.
    pub fn register(name: &str) -> Sel {
        let name = CString::from_slice(name.as_bytes());
        unsafe {
            sel_registerName(name.as_ptr())
        }
    }

    /// Returns the name of the method specified by self.
    pub fn name(&self) -> &str {
        unsafe {
            let name = sel_getName(*self);
            from_c_str(name)
        }
    }
}

impl PartialEq for Sel {
    fn eq(&self, other: &Sel) -> bool {
        self.ptr == other.ptr
    }
}

impl Eq for Sel { }

impl Copy for Sel { }

impl Clone for Sel {
    fn clone(&self) -> Sel { *self }
}

impl Ivar {
    /// Returns the name of self.
    pub fn name(&self) -> &str {
        unsafe {
            let name = ivar_getName(self);
            from_c_str(name)
        }
    }

    /// Returns the offset of self.
    pub fn offset(&self) -> isize {
        let offset = unsafe {
            ivar_getOffset(self)
        };
        offset as isize
    }

    /// Returns the type string of self.
    pub fn type_encoding(&self) -> &str {
        unsafe {
            let encoding = ivar_getTypeEncoding(self);
            from_c_str(encoding)
        }
    }
}

impl Method {
    /// Returns the name of self.
    pub fn name(&self) -> Sel {
        unsafe {
            method_getName(self)
        }
    }

    /// Returns a string describing self's parameter and return types.
    pub fn type_encoding(&self) -> &str {
        unsafe {
            let encoding = method_getTypeEncoding(self);
            from_c_str(encoding)
        }
    }

    /// Returns a string describing self's return type.
    pub fn return_type(&self) -> MallocString {
        unsafe {
            let encoding = method_copyReturnType(self);
            MallocString::new(encoding).unwrap()
        }
    }

    /// Returns a string describing a single parameter type of self, or
    /// `None` if self has no parameter at the given index.
    pub fn argument_type(&self, index: usize) -> Option<MallocString> {
        unsafe {
            let encoding = method_copyArgumentType(self, index as c_uint);
            if encoding.is_null() {
                None
            } else {
                Some(MallocString::new(encoding).unwrap())
            }
        }
    }

    /// Returns the number of arguments accepted by self.
    pub fn arguments_count(&self) -> usize {
        unsafe {
            method_getNumberOfArguments(self) as usize
        }
    }

    /// Returns the implementation of self.
    pub fn implementation(&self) -> Imp {
        unsafe {
            method_getImplementation(self)
        }
    }

    /// Sets the implementation of self.
    /// Unsafe because the caller must ensure the implementation has the
    /// correct self, return, and argument types for the method.
    pub unsafe fn set_implementation(&mut self, imp: Imp) -> Imp {
        method_setImplementation(self, imp)
    }

    /// Exchanges the implementations of self with another `Method`.
    /// Unsafe because the caller must ensure the two methods have the same
    /// self, return, and argument types.
    pub unsafe fn exchange_implementation(&mut self, other: &mut Method) {
        method_exchangeImplementations(self, other);
    }
}

impl Class {
    /// Returns the class definition of a specified class, or `None` if the
    /// class is not registered with the Objective-C runtime.
    pub fn get(name: &str) -> Option<&'static Class> {
        let name = CString::from_slice(name.as_bytes());
        unsafe {
            objc_getClass(name.as_ptr()).as_ref()
        }
    }

    /// Obtains the list of registered class definitions.
    pub fn classes() -> MallocBuffer<&'static Class> {
        unsafe {
            let mut count: c_uint = 0;
            let classes = objc_copyClassList(&mut count);
            MallocBuffer::new(classes as *mut _, count as usize).unwrap()
        }
    }

    /// Returns the total number of registered classes.
    pub fn classes_count() -> usize {
        unsafe {
            objc_getClassList(ptr::null_mut(), 0) as usize
        }
    }

    /// Returns the name of self.
    pub fn name(&self) -> &str {
        unsafe {
            let name = class_getName(self);
            from_c_str(name)
        }
    }

    /// Returns the size of instances of self.
    pub fn instance_size(&self) -> usize {
        unsafe {
            class_getInstanceSize(self) as usize
        }
    }

    /// Returns a specified instance method for self, or `None` if self and
    /// its superclasses do not contain an instance method with the
    /// specified selector.
    pub fn instance_method(&self, sel: Sel) -> Option<&Method> {
        unsafe {
            class_getInstanceMethod(self, sel).as_ref()
        }
    }

    /// Returns the ivar for a specified instance variable of self, or `None`
    /// if self has no ivar with the given name.
    pub fn instance_variable(&self, name: &str) -> Option<&Ivar> {
        let name = CString::from_slice(name.as_bytes());
        unsafe {
            class_getInstanceVariable(self, name.as_ptr()).as_ref()
        }
    }

    /// Describes the instance methods implemented by self.
    pub fn instance_methods(&self) -> MallocBuffer<&Method> {
        unsafe {
            let mut count: c_uint = 0;
            let methods = class_copyMethodList(self, &mut count);
            MallocBuffer::new(methods as *mut _, count as usize).unwrap()
        }

    }

    /// Describes the instance variables declared by self.
    pub fn instance_variables(&self) -> MallocBuffer<&Ivar> {
        unsafe {
            let mut count: c_uint = 0;
            let ivars = class_copyIvarList(self, &mut count);
            MallocBuffer::new(ivars as *mut _, count as usize).unwrap()
        }
    }
}

impl PartialEq for Class {
    fn eq(&self, other: &Class) -> bool {
        self as *const Class == other as *const Class
    }
}

impl Eq for Class { }

impl Object {
    /// Returns the class of self.
    pub fn class(&self) -> &'static Class {
        unsafe {
            &*object_getClass(self)
        }
    }

    /// Returns a reference to the ivar of self with the given name.
    /// Panics if self has no ivar with the given name.
    /// Unsafe because the caller must ensure that the ivar is actually
    /// of type `T`.
    pub unsafe fn get_ivar<T: Encode>(&self, name: &str) -> &T {
        let cls = self.class();
        let ptr = match cls.instance_variable(name) {
            Some(ivar) => {
                assert!(ivar.type_encoding() == encode::<T>());
                let offset = ivar.offset();
                let self_ptr = self as *const Object;
                (self_ptr as *const u8).offset(offset) as *const T
            }
            None => panic!("Ivar {} not found on class {}", name, cls.name()),
        };
        &*ptr
    }

    /// Returns a mutable reference to the ivar of self with the given name.
    /// Panics if self has no ivar with the given name.
    /// Unsafe because the caller must ensure that the ivar is actually
    /// of type `T`.
    pub unsafe fn get_mut_ivar<T: Encode>(&mut self, name: &str) -> &mut T {
        let cls = self.class();
        let ptr = match cls.instance_variable(name) {
            Some(ivar) => {
                assert!(ivar.type_encoding() == encode::<T>());
                let offset = ivar.offset();
                let self_ptr = self as *mut Object;
                (self_ptr as *mut u8).offset(offset) as *mut T
            }
            None => panic!("Ivar {} not found on class {}", name, cls.name()),
        };
        &mut *ptr
    }

    /// Sets the value of the ivar of self with the given name.
    /// Panics if self has no ivar with the given name.
    /// Unsafe because the caller must ensure that the ivar is actually
    /// of type `T`.
    pub unsafe fn set_ivar<T: Encode>(&mut self, name: &str, value: T) {
        *self.get_mut_ivar::<T>(name) = value;
    }
}

#[cfg(test)]
mod tests {
    use std::mem;
    use super::{Class, Sel};

    #[test]
    fn test_ivar() {
        let cls = Class::get("NSObject").unwrap();
        let ivar = cls.instance_variable("isa").unwrap();
        assert!(ivar.name() == "isa");
        assert!(ivar.type_encoding() == "#");
        assert!(ivar.offset() == 0);

        let ivars = cls.instance_variables();
        assert!(ivars.as_slice().len() > 0);
    }

    #[test]
    fn test_method() {
        let cls = Class::get("NSObject").unwrap();
        let sel = Sel::register("description");
        let method = cls.instance_method(sel).unwrap();
        assert!(method.name().name() == "description");
        assert!(method.type_encoding() != "");
        assert!(method.arguments_count() == 2);
        assert!(method.return_type().as_slice() == "@");
        assert!(method.argument_type(1).unwrap().as_slice() == ":");

        let methods = cls.instance_methods();
        assert!(methods.as_slice().len() > 0);
    }

    #[test]
    fn test_class() {
        let cls = Class::get("NSObject").unwrap();
        assert!(cls.name() == "NSObject");
        assert!(cls.instance_size() == mem::size_of::<*const Class>());
    }

    #[test]
    fn test_classes() {
        let classes_count = Class::classes_count();
        assert!(classes_count > 0);

        let classes = Class::classes();
        assert!(classes.as_slice().len() == classes_count);
    }

    #[test]
    fn test_object() {
        let cls = Class::get("NSObject").unwrap();
        let obj = unsafe {
            let obj = msg_send![cls, alloc];
            let obj = msg_send![obj, init];
            &*obj
        };
        assert!(obj.class() == cls);
        unsafe {
            msg_send![obj, release];
        }
    }
}

use runtime::{Message, Object};

pub trait ToMessage {
	fn as_ptr(&self) -> *mut Object;

	fn is_nil(&self) -> bool {
		self.as_ptr().is_null()
	}
}

impl<T: Message> ToMessage for *const T {
	fn as_ptr(&self) -> *mut Object {
		*self as *mut Object
	}
}

impl<T: Message> ToMessage for *mut T {
	fn as_ptr(&self) -> *mut Object {
		*self as *mut Object
	}
}

impl<'a, T: Message> ToMessage for &'a T {
	fn as_ptr(&self) -> *mut Object {
		(*self as *const T).as_ptr()
	}
}

impl<'a, T: Message> ToMessage for &'a mut T {
	fn as_ptr(&self) -> *mut Object {
		(*self as *mut T).as_ptr()
	}
}

impl<'a, T: Message> ToMessage for Option<&'a T> {
	fn as_ptr(&self) -> *mut Object {
		match self {
			&None => RawPtr::null(),
			&Some(ref obj) => obj.as_ptr(),
		}
	}
}

impl<'a, T: Message> ToMessage for Option<&'a mut T> {
	fn as_ptr(&self) -> *mut Object {
		match self {
			&None => RawPtr::null(),
			&Some(ref obj) => obj.as_ptr(),
		}
	}
}

pub fn to_ptr<T: ToMessage>(obj_ref: &T) -> *mut Object {
	obj_ref.as_ptr()
}

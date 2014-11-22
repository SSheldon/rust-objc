use runtime::{Message, Object};

pub trait ToMessage<T: Message> {
	fn as_ptr(&self) -> *mut T;

	fn is_nil(&self) -> bool {
		self.as_ptr().is_null()
	}
}

impl<T: Message> ToMessage<T> for *const T {
	fn as_ptr(&self) -> *mut T {
		*self as *mut T
	}
}

impl<T: Message> ToMessage<T> for *mut T {
	fn as_ptr(&self) -> *mut T {
		*self
	}
}

impl<'a, T: Message> ToMessage<T> for &'a T {
	fn as_ptr(&self) -> *mut T {
		*self as *const T as *mut T
	}
}

impl<'a, T: Message> ToMessage<T> for &'a mut T {
	fn as_ptr(&self) -> *mut T {
		*self
	}
}

impl<'a, T: Message> ToMessage<T> for Option<&'a T> {
	fn as_ptr(&self) -> *mut T {
		match *self {
			None => RawPtr::null(),
			Some(ref obj) => obj.as_ptr(),
		}
	}
}

impl<'a, T: Message> ToMessage<T> for Option<&'a mut T> {
	fn as_ptr(&self) -> *mut T {
		match *self {
			None => RawPtr::null(),
			Some(ref obj) => obj.as_ptr(),
		}
	}
}

pub fn to_ptr<T: Message, M: ToMessage<T>>(obj_ref: &M) -> *mut Object {
	obj_ref.as_ptr() as *mut Object
}
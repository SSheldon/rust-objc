#![macro_escape]

#[macro_export]
macro_rules! msg_send(
	($obj:expr $name:ident) => ({
		let sel_name = stringify!($name);
		let sel = ::runtime::Sel::register(sel_name);
		::runtime::objc_msgSend($obj, sel)
	});
	($obj:expr $($name:ident : $arg:expr)+) => ({
		let sel_name = concat!($(stringify!($name), ':'),+);
		let sel = ::runtime::Sel::register(sel_name);
		::runtime::objc_msgSend($obj, sel $(,$arg)+)
	});
)

#[macro_export]
macro_rules! object_struct(
	($name:ident<$($t:ident),+>) => (
		object_struct!($name, $($t),+)
	);
	($name:ident $(,$t:ident)*) => (
		pub struct $name<$($t),*> {
			ptr: ::id::Id,
		}

		impl<$($t),*> ::runtime::Messageable for $name<$($t),*> {
			unsafe fn as_ptr(&self) -> *::runtime::Object {
				use runtime::Messageable;
				self.ptr.as_ptr()
			}
		}

		impl<$($t),*> ::id::FromId for $name<$($t),*> {
			unsafe fn from_id(id: ::id::Id) -> $name<$($t),*> {
				$name { ptr: id }
			}
		}

		impl<$($t),*> ::foundation::INSObject for $name<$($t),*> {
			fn class_name() -> ::id::ClassName<$name<$($t),*>> {
				::id::ClassName::from_str(stringify!($name))
			}
		}

		impl<$($t),*> ::std::clone::Clone for $name<$($t),*> {
			fn clone(&self) -> $name<$($t),*> {
				unsafe {
					::id::FromId::from_id(self.ptr.clone())
				}
			}
		}

		impl<$($t),*> ::std::cmp::PartialEq for $name<$($t),*> {
			fn eq(&self, other: &$name<$($t),*>) -> bool {
				use foundation::INSObject;
				self.is_equal(other)
			}
		}

		impl<$($t),*> ::std::cmp::Eq for $name<$($t),*> { }

		impl<$($t,)* S: ::std::hash::Writer> ::std::hash::Hash<S> for $name<$($t),*> {
			fn hash(&self, state: &mut S) {
				use foundation::INSObject;
				self.hash_code().hash(state);
			}
		}

		impl<$($t),*> ::std::fmt::Show for $name<$($t),*> {
			fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
				use foundation::{INSObject, INSString};
				self.description().as_str().fmt(f)
			}
		}
	);
)

#![macro_escape]

#[macro_export]
macro_rules! object_struct(
	($name:ident) => (
		pub struct $name {
			ptr: ::id::Id,
		}

		impl ::runtime::Messageable for $name {
			unsafe fn as_ptr(&self) -> *::runtime::Object {
				use runtime::Messageable;
				self.ptr.as_ptr()
			}
		}

		impl ::id::FromId for $name {
			unsafe fn from_id(id: ::id::Id) -> $name {
				$name { ptr: id }
			}
		}

		impl ::foundation::INSObject for $name {
			fn class_name() -> ::id::ClassName<$name> {
				::id::ClassName::from_str(stringify!($name))
			}
		}

		impl ::std::clone::Clone for $name {
			fn clone(&self) -> $name {
				unsafe {
					::id::FromId::from_id(self.ptr.clone())
				}
			}
		}

		impl ::std::cmp::PartialEq for $name {
			fn eq(&self, other: &$name) -> bool {
				use foundation::INSObject;
				self.is_equal(other)
			}
		}

		impl ::std::cmp::Eq for $name { }

		impl<S: ::std::hash::Writer> ::std::hash::Hash<S> for $name {
			fn hash(&self, state: &mut S) {
				self.hash_code().hash(state);
			}
		}

		impl ::std::fmt::Show for $name {
			fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
				use foundation::{INSObject, INSString};
				self.description().as_str().fmt(f)
			}
		}
	);
)

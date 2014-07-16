#![macro_escape]

#[macro_export]
macro_rules! msg_send(
	($obj:expr $name:ident) => ({
		use runtime::Message;
		let sel_name = stringify!($name);
		let sel = ::runtime::Sel::register(sel_name);
		::runtime::objc_msgSend($obj.as_ptr(), sel)
	});
	($obj:expr $($name:ident : $arg:expr)+) => ({
		use runtime::Message;
		let sel_name = concat!($(stringify!($name), ':'),+);
		let sel = ::runtime::Sel::register(sel_name);
		::runtime::objc_msgSend($obj.as_ptr(), sel $(,$arg)+)
	});
)

#[macro_export]
macro_rules! object_struct(
	($name:ident<$($t:ident),+>) => (
		object_struct!($name, $($t),+)
	);
	($name:ident $(,$t:ident)*) => (
		pub struct $name<$($t),*> {
			nocopy: ::std::kinds::marker::NoCopy,
		}

		impl<$($t),*> ::runtime::Message for $name<$($t),*> {
			fn as_ptr(&self) -> *::runtime::Object {
				(self as *$name<$($t),*>) as *::runtime::Object
			}
		}

		impl<$($t),*> ::foundation::INSObject for $name<$($t),*> {
			fn class_name() -> ::ClassName<$name<$($t),*>> {
				::ClassName::from_str(stringify!($name))
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

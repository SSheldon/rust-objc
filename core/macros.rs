#![macro_escape]

#[macro_export]
macro_rules! msg_send(
	($obj:expr $name:ident) => ({
		let sel_name = stringify!($name);
		let sel = ::objc::runtime::Sel::register(sel_name);
		let ptr = ::objc::to_ptr(&$obj);
		::objc::runtime::objc_msgSend(ptr, sel)
	});
	($obj:expr $($name:ident : $arg:expr)+) => ({
		let sel_name = concat!($(stringify!($name), ':'),+);
		let sel = ::objc::runtime::Sel::register(sel_name);
		let ptr = ::objc::to_ptr(&$obj);
		::objc::runtime::objc_msgSend(ptr, sel $(,$arg)+)
	});
)

#[macro_export]
macro_rules! encode_message_impl(
	($code:expr, $name:ident $(,$t:ident)*) => (
		impl<'a $(, $t)*> ::objc::Encode for &'a $name<$($t),*> {
			fn code() -> ::objc::Encoding<&'a $name<$($t),*>> {
				::objc::Encoding($code)
			}
		}

		impl<'a $(, $t)*> ::objc::Encode for &'a mut $name<$($t),*> {
			fn code() -> ::objc::Encoding<&'a mut $name<$($t),*>> {
				::objc::Encoding($code)
			}
		}

		impl<'a $(, $t)*> ::objc::Encode for Option<&'a $name<$($t),*>> {
			fn code() -> ::objc::Encoding<Option<&'a $name<$($t),*>>> {
				::objc::Encoding($code)
			}
		}

		impl<'a $(, $t)*> ::objc::Encode for Option<&'a mut $name<$($t),*>> {
			fn code() -> ::objc::Encoding<Option<&'a mut $name<$($t),*>>> {
				::objc::Encoding($code)
			}
		}

		impl<$($t),*> ::objc::Encode for *const $name<$($t),*> {
			fn code() -> ::objc::Encoding<*const $name<$($t),*>> {
				::objc::Encoding($code)
			}
		}

		impl<$($t),*> ::objc::Encode for *mut $name<$($t),*> {
			fn code() -> ::objc::Encoding<*mut $name<$($t),*>> {
				::objc::Encoding($code)
			}
		}
	);
)

#[macro_export]
macro_rules! method(
	// Void no arguments
	(
		($self_ty:ty)$self_name:ident
		, $name:ident;
		$body:block
	) => ({
		method!(, stringify!($name), $body, $name, (), $self_name: $self_ty,)
	});
	// No arguments
	(
		($self_ty:ty)$self_name:ident
		, $name:ident
		-> $ret_ty:ty $body:block
	) => ({
		method!(, stringify!($name), $body, $name, $ret_ty, $self_name: $self_ty,)
	});
	// Void with arguments
	(
		($self_ty:ty)$self_name:ident
		, $name:ident : ($first_arg_ty:ty) $first_arg_name:ident
		$(, $next_name:ident : ($next_arg_ty:ty) $next_arg_name:ident)*;
		$body:block
	) => ({
		let sel_name = concat!(stringify!($name), ':', $(stringify!($next_name), ':'),*);
		method!(, sel_name, $body, $name, (), $self_name: $self_ty,
			$first_arg_name: $first_arg_ty$(, $next_arg_name: $next_arg_ty)*)
	});
	// Arguments
	(
		($self_ty:ty)$self_name:ident
		, $name:ident : ($first_arg_ty:ty) $first_arg_name:ident
		$(, $next_name:ident : ($next_arg_ty:ty) $next_arg_name:ident)*
		-> $ret_ty:ty $body:block
	) => ({
		let sel_name = concat!(stringify!($name), ':', $(stringify!($next_name), ':'),*);
		method!(, sel_name, $body, $name, $ret_ty, $self_name: $self_ty,
			$first_arg_name: $first_arg_ty$(, $next_arg_name: $next_arg_ty)*)
	});
	// Preceding comma is necessary to disambiguate
	(, $sel_name:expr, $body:block, $fn_name:ident, $ret_ty:ty, $self_name:ident : $self_ty:ty, $($arg_name:ident : $arg_ty:ty),*) => ({
		let sel = ::objc::runtime::Sel::register($sel_name);

		#[allow(non_snake_case)]
		extern fn $fn_name($self_name: $self_ty, _cmd: ::objc::runtime::Sel $(, $arg_name: $arg_ty)*) -> $ret_ty $body
		let imp: ::objc::runtime::Imp = unsafe { ::std::mem::transmute($fn_name) };

		let mut types = ::objc::encode::<$ret_ty>().to_string();
		types.push_str(::objc::encode::<$self_ty>());
		types.push_str(::objc::encode::<::objc::runtime::Sel>());
		$(types.push_str(::objc::encode::<$arg_ty>());)*

		::objc::MethodDecl { sel: sel, imp: imp, types: types }
	});
)

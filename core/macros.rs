#![macro_escape]

/// Sends a message to an object. The first argument should implement the
/// `ToMessage` trait, and the syntax is similar to the message syntax in
/// Objective-C.
///
/// Variadic arguments are not currently supported. This macro should only be
/// used in cases where `objc_msgSend` would be used, as opposed to
/// `objc_msgSend_stret` or `objc_msgSend_fpret`.
/// For more information, see Apple's documenation:
/// https://developer.apple.com/library/mac/documentation/Cocoa/Reference/ObjCRuntimeRef/index.html#//apple_ref/doc/uid/TP40001418-CH1g-88778
///
/// # Example
/// ```
/// let description = msg_send![obj description];
/// msg_send![obj setArg1:1u arg2:2u];
/// ```
#[macro_export]
macro_rules! msg_send(
    ($obj:expr $name:ident) => ({
        let sel_name = stringify!($name);
        let sel = ::objc::runtime::Sel::register(sel_name);
        let ptr = ::objc::to_obj_ptr(&$obj);
        ::objc::runtime::objc_msgSend(ptr, sel)
    });
    ($obj:expr $($name:ident : $arg:expr)+) => ({
        let sel_name = concat!($(stringify!($name), ':'),+);
        let sel = ::objc::runtime::Sel::register(sel_name);
        let ptr = ::objc::to_obj_ptr(&$obj);
        ::objc::runtime::objc_msgSend(ptr, sel $(,$arg)+)
    });
)

/// Implements the `Encode` trait for a `Message` type.
/// Specifically, this will implement `Encode` for reference, pointers, and
/// `Option` references of the given type.
///
/// The first argument should be a static string that is the type encoding
/// to use in the implementation. The second argument is the ident of the name
/// of the type to implement `Encode` for, and any further arguments are
/// used as type parameters for the type.
///
/// # Example
/// ```
/// impl Message for Object { }
/// encode_message_impl!("@", Object)
///
/// impl<T> Message for Array<T> { }
/// encode_message_impl!("@", Array, T)
/// ```
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

/// Declares a method, returning a `MethodDecl`.
/// The syntax is a combination of Objective-C's syntax and Rust's:
///
/// * The first part is the type and name of the self variable for the method
/// followed by a comma, like: `(&MyObject)this,`.
/// * Then, the parts of the selector and arguments follow, separated by commas,
/// like `setArg1:(uint)arg1, arg2:(uint)arg2`.
/// * After this, if the method has a return type, it should be followed by an
/// arrow ("->"), otherwise it should be terminated by a semicolon.
/// * Then a block declares the implementation of the method using the provided
/// arguments.
///
/// # Example
/// ```
/// method!(
///     (&mut MYObject)this, setNumber:(uint)number; {
///         this.set_number(number);
///     }
/// )
///
/// method!(
///     (&MYObject)this, number -> uint {
///         this.number()
///     }
/// )
/// ```
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

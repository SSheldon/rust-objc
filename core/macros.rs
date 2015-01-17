/// Registers a selector, returning an `Sel`.
///
/// # Example
/// ```
/// let sel = sel!(description);
/// let sel = sel!(setObject:forKey:);
/// ```
#[macro_export]
macro_rules! sel {
    // Declare a function to hide unsafety, otherwise we can trigger the
    // unused_unsafe lint; see rust-lang/rust#8472
    ($name:ident) => ({
        #[inline(always)]
        fn register_sel(name_with_nul: &str) -> $crate::runtime::Sel {
            let ptr = name_with_nul.as_ptr() as *const i8;
            unsafe { $crate::runtime::sel_registerName(ptr) }
        }
        register_sel(concat!(stringify!($name), '\0'))

    });
    ($($name:ident :)+) => ({
        #[inline(always)]
        fn register_sel(name_with_nul: &str) -> $crate::runtime::Sel {
            let ptr = name_with_nul.as_ptr() as *const i8;
            unsafe { $crate::runtime::sel_registerName(ptr) }
        }
        register_sel(concat!($(stringify!($name), ':'),+, '\0'))
    });
}

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
/// let description = msg_send![obj, description];
/// msg_send![obj, setArg1:1u arg2:2u];
/// ```
#[macro_export]
macro_rules! msg_send {
    ($obj:expr, $name:ident) => ({
        let sel = sel!($name);
        let ptr = $crate::ToMessage::as_ptr(&$obj) as *mut $crate::runtime::Object;
        $crate::runtime::objc_msgSend(ptr, sel)
    });
    ($obj:expr, $($name:ident : $arg:expr)+) => ({
        let sel = sel!($($name:)+);
        let ptr = $crate::ToMessage::as_ptr(&$obj) as *mut $crate::runtime::Object;
        $crate::runtime::objc_msgSend(ptr, sel $(,$arg)+)
    });
}

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
macro_rules! encode_message_impl {
    ($code:expr, $name:ident) => (
        encode_message_impl!($code, $name,);
    );
    ($code:expr, $name:ident, $($t:ident),*) => (
        impl<'a $(, $t)*> $crate::Encode for &'a $name<$($t),*> {
            fn code() -> $crate::Encoding<&'a $name<$($t),*>> {
                $crate::Encoding($code)
            }
        }

        impl<'a $(, $t)*> $crate::Encode for &'a mut $name<$($t),*> {
            fn code() -> $crate::Encoding<&'a mut $name<$($t),*>> {
                $crate::Encoding($code)
            }
        }

        impl<'a $(, $t)*> $crate::Encode for Option<&'a $name<$($t),*>> {
            fn code() -> $crate::Encoding<Option<&'a $name<$($t),*>>> {
                $crate::Encoding($code)
            }
        }

        impl<'a $(, $t)*> $crate::Encode for Option<&'a mut $name<$($t),*>> {
            fn code() -> $crate::Encoding<Option<&'a mut $name<$($t),*>>> {
                $crate::Encoding($code)
            }
        }

        impl<$($t),*> $crate::Encode for *const $name<$($t),*> {
            fn code() -> $crate::Encoding<*const $name<$($t),*>> {
                $crate::Encoding($code)
            }
        }

        impl<$($t),*> $crate::Encode for *mut $name<$($t),*> {
            fn code() -> $crate::Encoding<*mut $name<$($t),*>> {
                $crate::Encoding($code)
            }
        }
    );
}

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
///     (&MYObject)this, number -> uint, {
///         this.number()
///     }
/// )
/// ```
#[macro_export]
macro_rules! method {
    // Void no arguments
    (
        ($self_ty:ty)$self_name:ident,
        $name:ident
        ; $body:block
    ) => ({
        method!(-$name,; sel!($name), $body, (), $self_name: $self_ty,)
    });
    // No arguments
    (
        ($self_ty:ty)$self_name:ident,
        $name:ident
        -> $ret_ty:ty, $body:block
    ) => ({
        method!(-$name,; sel!($name), $body, $ret_ty, $self_name: $self_ty,)
    });
    // Void with arguments
    (
        ($self_ty:ty)$self_name:ident,
        $($name:ident : ($arg_ty:ty) $arg_name:ident),+
        ; $body:block
    ) => ({
        method!($(-$name,)+; sel!($($name:)+), $body, (), $self_name: $self_ty, $($arg_name: $arg_ty),+)
    });
    // Arguments
    (
        ($self_ty:ty)$self_name:ident,
        $($name:ident : ($arg_ty:ty) $arg_name:ident),+
        -> $ret_ty:ty, $body:block
    ) => ({
        method!($(-$name,)+; sel!($($name:)+), $body, $ret_ty, $self_name: $self_ty, $($arg_name: $arg_ty),+)
    });
    (
        // Preceding dash is necessary to disambiguate
        -$first_name:ident, $(-$next_name:ident,)*;
        $sel:expr, $body:block, $ret_ty:ty,
        $self_name:ident : $self_ty:ty, $($arg_name:ident : $arg_ty:ty),*
    ) => ({
        #[allow(non_snake_case)]
        extern fn $first_name($self_name: $self_ty, _cmd: $crate::runtime::Sel $(, $arg_name: $arg_ty)*) -> $ret_ty $body
        let imp: $crate::runtime::Imp = unsafe { ::std::mem::transmute($first_name) };

        let mut types = $crate::encode::<$ret_ty>().to_string();
        types.push_str($crate::encode::<$self_ty>());
        types.push_str($crate::encode::<$crate::runtime::Sel>());
        $(types.push_str($crate::encode::<$arg_ty>());)*

        $crate::MethodDecl { sel: $sel, imp: imp, types: types }
    });
}

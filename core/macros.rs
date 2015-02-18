/**
Registers a selector, returning an `Sel`.

# Example
```
# #[macro_use] extern crate objc;
# fn main() {
let sel = sel!(description);
let sel = sel!(setObject:forKey:);
# }
```
*/
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

/**
Sends a message to an object. The first argument should implement the
`ToMessage` trait, and the syntax is similar to the message syntax in
Objective-C. Variadic arguments are not currently supported.

# Example
``` no_run
# #[macro_use] extern crate objc;
# use objc::runtime::Object;
# fn main() {
# unsafe {
let obj: *mut Object;
# let obj: *mut Object = 0 as *mut Object;
let description: *const Object = msg_send![obj, description];
let _: () = msg_send![obj, setArg1:1 arg2:2];
# }
# }
```
*/
#[macro_export]
macro_rules! msg_send {
    (super($obj:expr, $superclass:expr), $name:ident) => ({
        let sel = sel!($name);
        $crate::send_super_message(&$obj, $superclass, sel, ())
    });
    (super($obj:expr, $superclass:expr), $($name:ident : $arg:expr)+) => ({
        let sel = sel!($($name:)+);
        $crate::send_super_message(&$obj, $superclass, sel, ($($arg,)*))
    });
    ($obj:expr, $name:ident) => ({
        let sel = sel!($name);
        $crate::send_message(&$obj, sel, ())
    });
    ($obj:expr, $($name:ident : $arg:expr)+) => ({
        let sel = sel!($($name:)+);
        $crate::send_message(&$obj, sel, ($($arg,)*))
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
/// ``` ignore
/// impl Message for Object { }
/// encode_message_impl!("@", Object)
///
/// impl<T> Message for Array<T> { }
/// encode_message_impl!("@", Array, T)
/// ```
macro_rules! encode_message_impl {
    ($code:expr, $name:ident) => (
        encode_message_impl!($code, $name,);
    );
    ($code:expr, $name:ident, $($t:ident),*) => (
        impl<'a $(, $t)*> $crate::Encode for &'a $name<$($t),*> {
            fn code() -> &'static str { $code }
        }

        impl<'a $(, $t)*> $crate::Encode for &'a mut $name<$($t),*> {
            fn code() -> &'static str { $code }
        }

        impl<'a $(, $t)*> $crate::Encode for Option<&'a $name<$($t),*>> {
            fn code() -> &'static str { $code }
        }

        impl<'a $(, $t)*> $crate::Encode for Option<&'a mut $name<$($t),*>> {
            fn code() -> &'static str { $code }
        }

        impl<$($t),*> $crate::Encode for *const $name<$($t),*> {
            fn code() -> &'static str { $code }
        }

        impl<$($t),*> $crate::Encode for *mut $name<$($t),*> {
            fn code() -> &'static str { $code }
        }
    );
}

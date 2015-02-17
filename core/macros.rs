/// Registers a selector, returning an `Sel`.
///
/// # Example
/// ``` ignore
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
/// ``` ignore
/// let description = msg_send![obj, description];
/// msg_send![obj, setArg1:1u arg2:2u];
/// ```
#[macro_export]
macro_rules! msg_send {
    ($obj:expr, $name:ident) => ({
        let obj = &$obj;
        let sel = sel!($name);
        let args = ();
        $crate::send_message(obj, sel, args)
    });
    ($obj:expr, $($name:ident : $arg:expr)+) => ({
        let obj = &$obj;
        let sel = sel!($($name:)+);
        let args = ($($arg,)*);
        $crate::send_message(obj, sel, args)
    });
}

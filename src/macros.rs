/**
Registers a selector, returning a `Sel`.

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
            let ptr = name_with_nul.as_ptr() as *const _;
            unsafe { $crate::runtime::sel_registerName(ptr) }
        }
        register_sel(concat!(stringify!($name), '\0'))
    });
    ($($name:ident :)+) => ({
        #[inline(always)]
        fn register_sel(name_with_nul: &str) -> $crate::runtime::Sel {
            let ptr = name_with_nul.as_ptr() as *const _;
            unsafe { $crate::runtime::sel_registerName(ptr) }
        }
        register_sel(concat!($(stringify!($name), ':'),+, '\0'))
    });
}

/**
Sends a message to an object.

The first argument can be any type that dereferences to a type that implements
`Message`, like a reference, pointer, or an `Id`.
The syntax is similar to the message syntax in Objective-C.
Variadic arguments are not currently supported.

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
        match $crate::__send_super_message(&*$obj, $superclass, sel, ()) {
            Err(s) => panic!("{}", s),
            Ok(r) => r,
        }
    });
    (super($obj:expr, $superclass:expr), $($name:ident : $arg:expr)+) => ({
        let sel = sel!($($name:)+);
        match $crate::__send_super_message(&*$obj, $superclass, sel, ($($arg,)*)) {
            Err(s) => panic!("{}", s),
            Ok(r) => r,
        }
    });
    ($obj:expr, $name:ident) => ({
        let sel = sel!($name);
        match $crate::__send_message(&*$obj, sel, ()) {
            Err(s) => panic!("{}", s),
            Ok(r) => r,
        }
    });
    ($obj:expr, $($name:ident : $arg:expr)+) => ({
        let sel = sel!($($name:)+);
        match $crate::__send_message(&*$obj, sel, ($($arg,)*)) {
            Err(s) => panic!("{}", s),
            Ok(r) => r,
        }
    });
}

#[cfg(feature = "exception")]
macro_rules! objc_try {
    ($b:block) => (
        match $crate::exception::try(|| $b) {
            Ok(result) => result,
            Err(Some(exception)) => panic!("Uncaught exception {:?}", &*exception),
            Err(None) => panic!("Uncaught exception nil"),
        }
    )
}

#[cfg(not(feature = "exception"))]
macro_rules! objc_try {
    ($b:block) => ($b)
}

macro_rules! count_idents {
    () => (0);
    ($a:ident) => (1);
    ($a:ident, $($b:ident),+) => (1 + count_idents!($($b),*));
}

macro_rules! encode {
    () => ("");
    (i8 $($x:tt)*) => (concat!("c", encode!($($x)*)));
    (i16 $($x:tt)*) => (concat!("s", encode!($($x)*)));
    (i32 $($x:tt)*) => (concat!("i", encode!($($x)*)));
    (i64 $($x:tt)*) => (concat!("q", encode!($($x)*)));
    (u8 $($x:tt)*) => (concat!("C", encode!($($x)*)));
    (u16 $($x:tt)*) => (concat!("S", encode!($($x)*)));
    (u32 $($x:tt)*) => (concat!("I", encode!($($x)*)));
    (u64 $($x:tt)*) => (concat!("Q", encode!($($x)*)));
    (f32 $($x:tt)*) => (concat!("f", encode!($($x)*)));
    (f64 $($x:tt)*) => (concat!("d", encode!($($x)*)));
    (bool $($x:tt)*) => (concat!("B", encode!($($x)*)));
    (Sel $($x:tt)*) => (concat!(":", encode!($($x)*)));
    (struct $i:ident {$($x:tt)+}) => (
        concat!('{', stringify!($i), '=', encode!($($x)*), '}')
    );
    // Just eat a leading comma and continue
    (, $($x:tt)*) => (encode!($($x)*));
}

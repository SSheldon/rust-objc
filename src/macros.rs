/**
Gets a reference to a `Class`.

Panics if no class with the given name can be found.
To check for a class that may not exist, use `Class::get`.

# Example
``` no_run
# #[macro_use] extern crate objc;
# fn main() {
let cls = class!(NSObject);
# }
```
*/
#[macro_export]
macro_rules! class {
    ($name:ident) => ({
        static CLASS: $crate::__CachedClass = $crate::__CachedClass::new();
        let name = concat!(stringify!($name), '\0');
        #[allow(unused_unsafe)]
        let cls = unsafe { CLASS.get(name) };
        match cls {
            Some(cls) => cls,
            None => panic!("Class with name {} could not be found", stringify!($name)),
        }
    })
}

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
    ($name:ident) => ({
        static SEL: $crate::__CachedSel = $crate::__CachedSel::new();
        let name = concat!(stringify!($name), '\0');
        #[allow(unused_unsafe)]
        unsafe { SEL.get(name) }
    });
    ($($name:ident :)+) => ({
        static SEL: $crate::__CachedSel = $crate::__CachedSel::new();
        let name = concat!($(stringify!($name), ':'),+, '\0');
        #[allow(unused_unsafe)]
        unsafe { SEL.get(name) }
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
# let obj = 0 as *mut Object;
let description = msg_send![obj, description => *mut Object];
msg_send![obj, setArg1:1 arg2:2 => ()];
# }
# }
```
*/
#[macro_export]
macro_rules! msg_send {
    (super($obj:expr, $superclass:expr), $name:ident => $ret:ty) => ({
        let sel = $crate::sel!($name);
        match $crate::__send_super_message::<_, _, $ret>(&*$obj, $superclass, sel, ()) {
            Err(s) => panic!("{}", s),
            Ok(r) => r,
        }
    });
    (super($obj:expr, $superclass:expr), $($name:ident : $arg:expr)+ => $ret:ty) => ({
        let sel = $crate::sel!($($name:)+);
        match $crate::__send_super_message::<_, _, $ret>(&*$obj, $superclass, sel, ($($arg,)*)) {
            Err(s) => panic!("{}", s),
            Ok(r) => r,
        }
    });
    ($obj:expr, $name:ident => $ret:ty) => ({
        let sel = $crate::sel!($name);
        match $crate::__send_message::<_, _, $ret>(&*$obj, sel, ()) {
            Err(s) => panic!("{}", s),
            Ok(r) => r,
        }
    });
    ($obj:expr, $($name:ident : $arg:expr)+ => $ret:ty) => ({
        let sel = $crate::sel!($($name:)+);
        match $crate::__send_message::<_, _, $ret>(&*$obj, sel, ($($arg,)*)) {
            Err(s) => panic!("{}", s),
            Ok(r) => r,
        }
    });
}

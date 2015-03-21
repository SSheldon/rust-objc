Objective-C Runtime bindings and wrapper for Rust.

* Documentation: http://ssheldon.github.io/rust-objc/objc/
* Crate: https://crates.io/crates/objc

## Messaging objects

Objective-C objects can be messaged using the `msg_send!` macro:

``` rust
let cls = Class::get("NSObject").unwrap();
let obj: *mut Object = msg_send![cls, new];
let hash: usize = msg_send![obj, hash];
let is_kind: BOOL = msg_send![obj, isKindOfClass:cls];
// Even void methods must have their return type annotated
let _: () = msg_send![obj, release];
```

## Reference counting

Objective-C objects are reference counted; to ensure that they are retained and
released at the proper times, we can use the `Id` struct.

To enforce aliasing rules, an `Id` can be either owned or shared; if it is
owned, meaning the `Id` is the only reference to the object, it can be mutably
dereferenced. An owned `Id` can be downgraded to a `ShareId`
which can be cloned to allow multiple references.

Weak references may be created using the `WeakId` struct.

``` rust
let cls = Class::get("NSObject").unwrap();
let obj: Id<Object> = unsafe {
    Id::from_retained_ptr(msg_send![cls, new])
};
// obj will be released when it goes out of scope

// share the object so we can clone it
let obj = obj.share();
let another_ref = obj.clone();
// dropping our other reference will decrement the retain count
drop(another_ref);

let weak = WeakId::new(&obj);
assert!(weak.load().is_some());
// After the object is deallocated, our weak pointer returns none
drop(obj);
assert!(weak.load().is_none());
```

## Invoking blocks

The `Block` struct is used for invoking blocks from Objective-C. For example,
consider this Objective-C function:

``` objc
int32_t sum(int32_t (^block)(int32_t, int32_t)) {
    return block(5, 8);
}
```

We could write it in Rust as the following:

``` rust
fn sum(block: &mut Block<(i32, i32), i32>) -> i32 {
    block.call((5, 8))
}
```

Note the extra parentheses in the `call` method, since the arguments must be
passed as a tuple.

## Creating blocks

Creating a block to pass to Objective-C can be done with the `ConcreteBlock`
struct. For example, to create a block that adds two `i32`s, we could write:

``` rust
let block = ConcreteBlock::new(|a: i32, b: i32| a + b);
let mut block = block.copy();
assert!(block.call((5, 8)) == 13);
```

It is important to copy your block to the heap (with the `copy` method) before
passing it to Objective-C; this is because our `ConcreteBlock` is only meant
to be copied once, and we can enforce this in Rust, but if Objective-C code
were to copy it twice we could have a double free.

## Declaring classes

Classes can be declared using the `ClassDecl` struct. Instance variables and
methods can then be added before the class is ultimately registered.

The following example demonstrates declaring a class named `MyNumber` that has
one ivar, a `u32` named `_number` and a `number` method that returns it:

``` rust
let superclass = Class::get("NSObject").unwrap();
let mut decl = ClassDecl::new(superclass, "MyNumber").unwrap();

// Add an instance variable
decl.add_ivar::<u32>("_number");

// Add an ObjC method for getting the number
extern fn my_number_get(this: &Object, _cmd: Sel) -> u32 {
    unsafe { *this.get_ivar("_number") }
}
unsafe {
    decl.add_method(sel!(number),
        my_number_get as extern fn(&Object, Sel) -> u32);
}

decl.register();
```

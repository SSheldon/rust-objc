#![feature(default_type_params, macro_rules, unsafe_destructor)]
#![allow(dead_code)]

use id::Id;
use foundation::{NSArray, NSDictionary, NSObject, NSString,
	INSArray, INSCopying, INSDictionary, INSObject, INSString};

mod macros;

mod runtime;
mod id;
mod foundation;

fn main() {
	let obj: Id<NSObject> = INSObject::new();
	let obj2 = obj.clone();

	println!("{} == {}? {}", obj, obj2, obj == obj2);

	let obj3: Id<NSObject> = INSObject::new();
	println!("{} == {}? {}", obj, obj3, obj == obj3);

	let objs = vec![obj.clone(), obj2.clone(), obj3.clone()];
	let array: Id<NSArray<NSObject>> = INSArray::from_vec(objs);
/*
	for obj in array.object_enumerator() {
		println!("{}", obj);
	}
*/
	println!("{}", array.len());

	let string: Id<NSString> = INSString::from_str("Hello, world!");
	println!("{}", string.as_str());
	let string2 = string.copy();
	println!("{}", string2.as_str());

	let keys = [&*string];
	let vals = vec![obj.clone()];
	let dict: Id<NSDictionary<NSString, NSObject>> =
		INSDictionary::from_keys_and_objects(keys.as_slice(), vals);
	println!("{}", dict.object_for(&*string));
	println!("{}", dict.len());
}

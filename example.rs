#![feature(default_type_params)]
#![allow(dead_code)]

use foundation::{NSArray, NSDictionary, NSObject, NSString,
	INSArray, INSCopying, INSDictionary, INSObject, INSString};

mod runtime;
mod id;
mod foundation;

fn main() {
	let obj: NSObject = INSObject::new();
	let obj2 = obj.clone();

	println!("{} == {}? {}", obj, obj2, obj == obj2);

	let obj3: NSObject = INSObject::new();
	println!("{} == {}? {}", obj, obj3, obj == obj3);

	let objs = [obj.clone(), obj2.clone(), obj3.clone()];
	let array = NSArray::from_slice(objs.as_slice());
	for obj in array.object_enumerator() {
		println!("{}", obj);
	}
	println!("{}", array.len());

	let string = NSString::from_str("Hello, world!");
	println!("{}", string.as_str());
	let string2 = string.copy();
	println!("{}", string2.as_str());

	let keys = [string.clone()];
	let vals = [obj.clone()];
	let dict = NSDictionary::from_keys_and_objects(keys.as_slice(), vals.as_slice());
	println!("{}", dict.object_for(&string));
	println!("{}", dict.len());
}

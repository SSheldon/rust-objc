use objc::runtime::{Class, Protocol, Sel};
use objc::{Encode, msg_send};

#[path = "common/mod.rs"]
mod test_utils;

#[test]
fn test_ivar() {
    let cls = test_utils::custom_class();
    let ivar = cls.instance_variable("_foo").unwrap();
    assert!(ivar.name() == "_foo");
    assert!(ivar.type_encoding() == &<u32>::ENCODING);
    assert!(ivar.offset() > 0);

    let ivars = cls.instance_variables();
    assert!(ivars.len() > 0);
}

#[test]
fn test_method() {
    let cls = test_utils::custom_class();
    let sel = Sel::register("foo");
    let method = cls.instance_method(sel).unwrap();
    assert!(method.name().name() == "foo");
    assert!(method.arguments_count() == 2);
    assert!(*method.return_type() == <u32>::ENCODING);
    assert!(*method.argument_type(1).unwrap() == Sel::ENCODING);

    let methods = cls.instance_methods();
    assert!(methods.len() > 0);
}

#[test]
fn test_class() {
    let cls = test_utils::custom_class();
    assert!(cls.name() == "CustomObject");
    assert!(cls.instance_size() > 0);
    assert!(cls.superclass().is_none());

    assert!(Class::get(cls.name()) == Some(cls));

    let metaclass = cls.metaclass();
    // The metaclass of a root class is a subclass of the root class
    assert!(metaclass.superclass().unwrap() == cls);

    let subclass = test_utils::custom_subclass();
    assert!(subclass.superclass().unwrap() == cls);
}

#[test]
fn test_classes() {
    assert!(Class::classes_count() > 0);
    let classes = Class::classes();
    assert!(classes.len() > 0);
}

#[test]
fn test_protocol() {
    let proto = test_utils::custom_protocol();
    assert!(proto.name() == "CustomProtocol");
    let class = test_utils::custom_class();
    assert!(class.conforms_to(proto));
    let class_protocols = class.adopted_protocols();
    assert!(class_protocols.len() > 0);
}

#[test]
fn test_protocol_method() {
    let class = test_utils::custom_class();
    let result: i32 = unsafe {
        msg_send![class, addNumber:1 toNumber:2]
    };
    assert_eq!(result, 3);
}

#[test]
fn test_subprotocols() {
    let sub_proto = test_utils::custom_subprotocol();
    let super_proto = test_utils::custom_protocol();
    assert!(sub_proto.conforms_to(super_proto));
    let adopted_protocols = sub_proto.adopted_protocols();
    assert_eq!(adopted_protocols[0], super_proto);
}

#[test]
fn test_protocols() {
    // Ensure that a protocol has been registered on linux
    let _ = test_utils::custom_protocol();

    let protocols = Protocol::protocols();
    assert!(protocols.len() > 0);
}

#[test]
fn test_object() {
    let mut obj = test_utils::custom_object();
    assert!(obj.class() == test_utils::custom_class());
    let result: u32 = unsafe {
        obj.set_ivar("_foo", 4u32);
        *obj.get_ivar("_foo")
    };
    assert!(result == 4);
}

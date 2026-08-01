use js_bindgen_test::test;
use js_sys::{Array, Function, JsString, JsValue, Object, PropertyDescriptor, Reflect, Symbol};

#[test]
fn properties() {
	let object = Object::new();
	let value = JsString::from("value");

	assert!(Reflect::set_str(object.as_ref(), "key", value.as_ref()).unwrap());
	assert!(Reflect::has_str(object.as_ref(), "key").unwrap());
	assert_eq!(
		&Reflect::get_str(object.as_ref(), "key").unwrap(),
		value.as_ref()
	);
	assert_eq!(Reflect::own_keys(object.as_ref()).unwrap().length(), 1);
	assert!(Reflect::delete_property_str(object.as_ref(), "key").unwrap());
	assert!(!Reflect::has_str(object.as_ref(), "key").unwrap());

	let descriptor = Object::new();
	assert!(Reflect::set_str(descriptor.as_ref(), "value", value.as_ref()).unwrap());
	assert!(Reflect::define_property_str(object.as_ref(), "fixed", descriptor.as_ref()).unwrap());
	assert_eq!(
		&Reflect::get_str(object.as_ref(), "fixed").unwrap(),
		value.as_ref()
	);
	assert_ne!(
		Reflect::get_own_property_descriptor_str(object.as_ref(), "fixed").unwrap(),
		JsValue::UNDEFINED
	);

	let symbol = Symbol::new();
	assert!(Reflect::set(object.as_ref(), symbol.as_ref(), value.as_ref()).unwrap());
	assert!(Reflect::has(object.as_ref(), symbol.as_ref()).unwrap());
	assert_eq!(
		Reflect::get(object.as_ref(), symbol.as_ref()).unwrap(),
		JsValue::from(value.clone())
	);
	assert!(Reflect::delete_property(object.as_ref(), symbol.as_ref()).unwrap());

	let array = Array::new();
	assert!(Reflect::set_u32(array.as_ref(), 2, value.as_ref()).unwrap());
	assert_eq!(
		Reflect::get_u32(array.as_ref(), 2).unwrap(),
		JsValue::from(value)
	);
}

#[test]
fn receiver_controls_accessor_this() {
	let target = Object::new();
	let receiver = Object::new();
	let property = JsValue::from(JsString::from("accessor"));
	let getter = Function::new_no_args("return this.stored").unwrap();
	let setter = Function::new_with_args("value", "this.stored = value").unwrap();
	let descriptor = PropertyDescriptor::new();
	descriptor.set_get(&getter);
	descriptor.set_set(&setter);
	Object::define_property(&target, &property, &descriptor).unwrap();

	let initial = JsString::from("initial");
	assert!(Reflect::set_str(receiver.as_ref(), "stored", initial.as_ref()).unwrap());
	assert_eq!(
		Reflect::get_with_receiver(target.as_ref(), &property, receiver.as_ref()).unwrap(),
		JsValue::from(initial)
	);

	let updated = JsString::from("updated");
	assert!(
		Reflect::set_with_receiver(
			target.as_ref(),
			&property,
			updated.as_ref(),
			receiver.as_ref(),
		)
		.unwrap()
	);
	assert_eq!(
		Reflect::get_str(receiver.as_ref(), "stored").unwrap(),
		JsValue::from(updated)
	);
}

#[test]
fn apply_and_construct() {
	let concatenate = Function::new_with_args("left, right", "return left + right").unwrap();
	let arguments: Array<JsString> = Array::new_typed();
	assert_eq!(arguments.push(&JsString::from("a")), 1);
	assert_eq!(arguments.push(&JsString::from("b")), 2);

	let result = Reflect::apply(&concatenate, &JsValue::UNDEFINED, arguments.as_untyped()).unwrap();
	assert_eq!(&result, JsString::from("ab").as_ref());

	let constructor = Function::new_with_args("value", "this.value = value").unwrap();
	let arguments: Array<JsString> = Array::new_typed();
	assert_eq!(arguments.push(&JsString::from("constructed")), 1);
	let object = Reflect::construct(&constructor, arguments.as_untyped()).unwrap();
	assert_eq!(
		&Reflect::get_str(&object, "value").unwrap(),
		JsString::from("constructed").as_ref()
	);
}

#[test]
fn prototypes_and_extensibility() {
	let object = Object::new();
	let prototype = Object::new();

	assert!(Reflect::set_prototype_of(object.as_ref(), prototype.as_ref()).unwrap());
	assert_eq!(
		&Reflect::get_prototype_of(object.as_ref()).unwrap(),
		prototype.as_ref()
	);
	assert!(Reflect::is_extensible(object.as_ref()).unwrap());
	assert!(Reflect::prevent_extensions(object.as_ref()).unwrap());
	assert!(!Reflect::is_extensible(object.as_ref()).unwrap());
	assert!(!Reflect::set_str(object.as_ref(), "new", &JsValue::NULL).unwrap());
}

#[test]
fn exceptions_are_returned() {
	assert!(Reflect::get(&JsValue::NULL, &JsValue::UNDEFINED).is_err());
}

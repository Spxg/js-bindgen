use js_bindgen_test::test;
use js_sys::{Array, Function, JsString, JsValue, Object, Reflect, Symbol};

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

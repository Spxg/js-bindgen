use js_bindgen_test::test;
use js_sys::{JsString, JsValue, Object, PropertyDescriptor, Reflect};

#[test]
fn properties_and_descriptors() {
	let object = Object::new();
	let name = JsValue::from(JsString::from("answer"));
	let value = JsValue::from(JsString::from("forty-two"));
	let descriptor = PropertyDescriptor::new();
	descriptor.set_value(&value);
	descriptor.set_enumerable(true);
	descriptor.set_writable(false);
	descriptor.set_configurable(true);

	Object::define_property(&object, &name, &descriptor).unwrap();
	assert!(Object::has_own(&object, &name).unwrap());
	assert_eq!(Reflect::get(object.as_ref(), &name).unwrap(), value);
	assert_eq!(Object::keys(&object).unwrap().length(), 1);
	assert_eq!(Object::values(&object).unwrap().length(), 1);
	assert_eq!(Object::entries(&object).unwrap().length(), 1);

	let descriptor = Object::get_own_property_descriptor(&object, &name)
		.unwrap()
		.unwrap();
	assert_eq!(descriptor.value(), value);
	assert_eq!(descriptor.enumerable(), Some(true));
	assert_eq!(descriptor.writable(), Some(false));
	assert_eq!(descriptor.configurable(), Some(true));
	assert!(object.has_own_property(&name).unwrap());
	assert!(object.property_is_enumerable(&name).unwrap());
}

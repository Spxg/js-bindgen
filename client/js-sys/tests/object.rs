use js_bindgen_test::test;
use js_sys::{
	Array, Function, JsString, JsValue, Object, PropertyDescriptor, Proxy, Reflect, Symbol,
};

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

#[test]
fn accessor_descriptors() {
	let object = Object::new();
	let name = JsValue::from(JsString::from("accessor"));
	let getter = Function::new_no_args("return this.stored").unwrap();
	let setter = Function::new_with_args("value", "this.stored = `set:${value}`").unwrap();
	let descriptor = PropertyDescriptor::new();
	descriptor.set_get(&getter);
	descriptor.set_set(&setter);
	descriptor.set_configurable(true);

	Object::define_property(&object, &name, &descriptor).unwrap();
	let input = JsString::from("input");
	assert!(Reflect::set(object.as_ref(), &name, input.as_ref()).unwrap());
	assert_eq!(
		Reflect::get_str(object.as_ref(), "stored").unwrap(),
		JsValue::from(JsString::from("set:input"))
	);
	assert_eq!(
		Reflect::get(object.as_ref(), &name).unwrap(),
		JsValue::from(JsString::from("set:input"))
	);

	let descriptor = Object::get_own_property_descriptor(&object, &name)
		.unwrap()
		.unwrap();
	assert!(descriptor.get().is_some());
	assert!(descriptor.set().is_some());
	assert_eq!(descriptor.value(), JsValue::UNDEFINED);
	assert_eq!(descriptor.writable(), None);
}

#[test]
fn construction_and_integrity() {
	let null_prototype = Object::create(&JsValue::NULL).unwrap();
	assert_eq!(
		Object::get_prototype_of(&null_prototype).unwrap(),
		JsValue::NULL
	);
	assert!(Object::is_extensible(&null_prototype).unwrap());

	Object::prevent_extensions(&null_prototype).unwrap();
	assert!(!Object::is_extensible(&null_prototype).unwrap());

	let frozen = Object::freeze(&Object::new()).unwrap();
	assert!(Object::is_frozen(&frozen).unwrap());
	let sealed = Object::seal(&Object::new()).unwrap();
	assert!(Object::is_sealed(&sealed).unwrap());

	assert!(Object::is(&JsValue::NULL, &JsValue::NULL));
	assert!(!Object::is(&JsValue::NULL, &JsValue::UNDEFINED));
}

#[test]
fn static_collection_methods() {
	let key = JsValue::from(JsString::from("key"));
	let value = JsValue::from(JsString::from("value"));
	let pair = Array::of(&[key.clone(), value.clone()]);
	let entries = Array::of(&[pair]);
	let object = Object::from_entries(entries.as_ref()).unwrap();
	assert_eq!(Reflect::get(object.as_ref(), &key).unwrap(), value);

	let target = Object::new();
	let assigned = Object::assign(&target, &object).unwrap();
	assert_eq!(Reflect::get(assigned.as_ref(), &key).unwrap(), value);

	let symbol = Symbol::new();
	Reflect::set(object.as_ref(), symbol.as_ref(), &JsValue::NULL).unwrap();
	assert_eq!(
		Object::get_own_property_symbols(&object).unwrap().length(),
		1
	);
	assert!(Object::get_own_property_names(&object).unwrap().length() >= 1);
}

#[test]
fn proxy_exceptions_are_returned() {
	let revocable = Proxy::revocable(Object::new().as_ref(), Object::new().as_ref()).unwrap();
	let proxy = revocable.proxy();
	revocable.revoke().call(&JsValue::UNDEFINED, &[]).unwrap();
	assert!(Object::keys(&proxy).is_err());
	assert!(Object::get_prototype_of(&proxy).is_err());
	assert!(proxy.has_own_property(&JsValue::NULL).is_err());
	assert!(proxy.to_string().is_err());
}

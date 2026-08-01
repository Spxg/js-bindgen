use js_bindgen_test::test;
use js_sys::{Boolean, JsString, JsValue, Object};

fn assert_object<T: AsRef<Object>>(_: &T) {}

#[test]
fn constructors_and_methods() {
	let default = Boolean::new();
	assert_object(&default);
	assert!(!default.value_of());
	assert_eq!(default.to_string(), "false");

	let false_value = Boolean::new_with_value(&JsValue::UNDEFINED);
	assert!(!false_value.value_of());

	let truthy = JsValue::from(JsString::from("false"));
	let true_value = Boolean::new_with_value(&truthy);
	assert!(true_value.value_of());
	assert_eq!(true_value.to_string(), "true");

	assert!(!Boolean::default().value_of());
}

#[test]
fn boxed_values_use_object_identity() {
	let truthy = JsValue::from(JsString::from("truthy"));
	let first = Boolean::new_with_value(&truthy);
	let second = Boolean::new_with_value(&truthy);

	assert_ne!(first, second);
	assert_eq!(first, first.clone());
	assert!(first.value_of());
	assert!(second.value_of());
}

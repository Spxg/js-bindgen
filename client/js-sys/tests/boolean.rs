use js_bindgen_test::test;
use js_sys::{Boolean, JsString, JsValue, Object};

fn assert_object<T: AsRef<Object>>(_: &T) {}

#[test]
fn constructor_and_instance_methods() {
	let default = Boolean::new();
	assert_object(&default);
	assert!(!default.value_of());

	let truthy = JsValue::from(JsString::from("false"));
	assert!(Boolean::new_with_value(&truthy).value_of());
}

use js_bindgen_test::test;
use js_sys::{Error, JsString, JsValue, Object, SuppressedError};

fn assert_parents<T: AsRef<Error> + AsRef<Object>>(_: &T) {}

#[test]
fn constructor_and_properties() {
	let error = JsValue::from(JsString::from("primary"));
	let suppressed = JsValue::from(JsString::from("secondary"));
	let value = SuppressedError::new(&error, &suppressed);

	assert_parents(&value);
	assert_eq!(value.error(), error);
	assert_eq!(value.suppressed(), suppressed);
	assert_eq!(value.name(), JsString::from("SuppressedError"));
	assert_eq!(value.message(), JsString::from(""));

	let replacement_error = JsValue::from(JsString::from("replacement primary"));
	let replacement_suppressed = JsValue::from(JsString::from("replacement secondary"));
	value.set_error(&replacement_error);
	value.set_suppressed(&replacement_suppressed);
	assert_eq!(value.error(), replacement_error);
	assert_eq!(value.suppressed(), replacement_suppressed);

	let described =
		SuppressedError::new_with_message(&JsValue::NULL, &JsValue::UNDEFINED, "cleanup failed");
	assert_eq!(described.message(), JsString::from("cleanup failed"));
}

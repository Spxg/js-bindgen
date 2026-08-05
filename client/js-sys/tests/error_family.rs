use js_bindgen_test::test;
use js_sys::{AggregateError, Error, ErrorOptions, JsString, JsValue, Object, TypeError};

fn assert_error(error: &Error, message: &str, name: &str, cause: &JsValue) {
	assert_eq!(error.message(), JsString::from(message));
	assert_eq!(error.name(), JsString::from(name));
	assert_eq!(error.cause(), cause.clone());
	let display = if message.is_empty() {
		name.to_owned()
	} else {
		format!("{name}: {message}")
	};
	assert_eq!(error.to_string(), JsString::from(display));
}

fn assert_parents<T: AsRef<Error> + AsRef<Object>>(_: &T) {}

#[test]
fn inherited_constructor_and_properties() {
	let cause = JsValue::from(JsString::from("root cause"));
	let options = ErrorOptions::new(&cause);
	let error = TypeError::new_with_options("failed", &options);
	assert_parents(&error);
	assert_error(&error, "failed", "TypeError", &cause);
	assert!(Error::is_error(error.as_ref()));
	error.set_message("updated");
	assert_eq!(error.message(), JsString::from("updated"));
}

#[test]
fn aggregate_error() {
	let value = JsValue::from(Error::new("inner"));
	let error = AggregateError::new_with_message(core::slice::from_ref(&value), "combined");
	assert_parents(&error);
	assert_eq!(error.errors().get(0), value);
	assert_eq!(error.message(), JsString::from("combined"));
}

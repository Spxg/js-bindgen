use js_bindgen_test::test;
use js_sys::{
	AggregateError, Array, Error, ErrorOptions, EvalError, JsString, JsValue, Object, RangeError,
	ReferenceError, SyntaxError, TypeError, UriError,
};

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
fn standard_error_types() {
	let cause = JsValue::from(JsString::from("root cause"));
	let options = ErrorOptions::new(&cause);

	macro_rules! assert_type {
		($type:ident, $name:literal) => {{
			let plain = $type::new("plain");
			assert_parents(&plain);
			assert_error(&plain, "plain", $name, &JsValue::UNDEFINED);

			let caused = $type::new_with_options("failed", &options);
			assert_error(&caused, "failed", $name, &cause);
		}};
	}

	assert_type!(EvalError, "EvalError");
	assert_type!(RangeError, "RangeError");
	assert_type!(ReferenceError, "ReferenceError");
	assert_type!(SyntaxError, "SyntaxError");
	assert_type!(TypeError, "TypeError");
	assert_type!(UriError, "URIError");
}

#[test]
fn branded_error_check() {
	let error = Error::new("error");
	assert!(Error::is_error(error.as_ref()));
	assert!(Error::is_error(TypeError::new("type error").as_ref()));
	assert!(!Error::is_error(Object::new().as_ref()));
	assert!(!Error::is_error(&JsValue::NULL));
}

#[test]
fn inherited_properties_are_writable() {
	let error = TypeError::new("before");
	assert_eq!(error.cause(), JsValue::UNDEFINED);

	error.set_message("after");
	error.set_name("CustomError");
	let cause = JsValue::from(JsString::from("cause"));
	error.set_cause(&cause);

	assert_error(&error, "after", "CustomError", &cause);
}

#[test]
fn aggregate_error() {
	let first = JsValue::from(JsString::from("first"));
	let second = JsValue::from(Error::new("second"));
	let values = [first.clone(), second.clone()];

	let plain = AggregateError::new(&values);
	assert_parents(&plain);
	assert_error(&plain, "", "AggregateError", &JsValue::UNDEFINED);
	assert_eq!(plain.errors().length(), 2);

	let described = AggregateError::new_with_message(&values, "combined");
	assert_error(
		&described,
		"combined",
		"AggregateError",
		&JsValue::UNDEFINED,
	);
	let errors = described.errors();
	assert_eq!(errors.get(0), first);
	assert_eq!(errors.get(1), second);

	let cause = JsValue::from(JsString::from("root cause"));
	let options = ErrorOptions::new(&cause);
	let caused = AggregateError::new_with_options(&values, "combined", &options);
	assert_error(&caused, "combined", "AggregateError", &cause);

	let replacement = Array::new();
	let replacement_value = JsValue::from(JsString::from("replacement"));
	let _ = replacement.push(&replacement_value);
	caused.set_errors(&replacement);
	assert_eq!(caused.errors().get(0), replacement_value);
}

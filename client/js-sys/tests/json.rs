use js_bindgen_test::test;
use js_sys::{Function, JSON, JsString, JsValue, Object, Reflect};

#[test]
fn parse_and_stringify() {
	let value = JSON::parse(r#"{"name":"js-bindgen"}"#).unwrap();
	assert_eq!(
		&Reflect::get_str(&value, "name").unwrap(),
		JsString::from("js-bindgen").as_ref()
	);
	assert_eq!(
		JSON::stringify(&value).unwrap().unwrap(),
		r#"{"name":"js-bindgen"}"#
	);
}

#[test]
fn optional_and_exception_results() {
	assert!(JSON::stringify(&JsValue::UNDEFINED).unwrap().is_none());
	assert!(JSON::parse("{").is_err());

	let cyclic = Object::new();
	Reflect::set_str(cyclic.as_ref(), "self", cyclic.as_ref()).unwrap();
	assert!(JSON::stringify(cyclic.as_ref()).is_err());
}

#[test]
fn reviver_and_formatting() {
	let reviver =
		Function::new_with_args("key, value", "return key === 'discard' ? undefined : value")
			.unwrap();
	let source = r#"{"keep":"yes","discard":"no"}"#;
	let value = JSON::parse_with_reviver(source, &reviver).unwrap();
	assert!(Reflect::has_str(&value, "keep").unwrap());
	assert!(!Reflect::has_str(&value, "discard").unwrap());

	let value = JSON::parse(source).unwrap();
	assert_eq!(
		JSON::stringify_with_replacer(&value, reviver.as_ref())
			.unwrap()
			.unwrap(),
		r#"{"keep":"yes"}"#
	);

	let object = Object::new();
	let string = JsString::from("value");
	Reflect::set_str(object.as_ref(), "key", string.as_ref()).unwrap();
	let space: JsValue = JsString::from("  ").into();
	let formatted =
		JSON::stringify_with_replacer_and_space(object.as_ref(), &JsValue::NULL, &space)
			.unwrap()
			.unwrap();
	assert_eq!(formatted, "{\n  \"key\": \"value\"\n}");
}

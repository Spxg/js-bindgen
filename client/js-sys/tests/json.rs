use js_bindgen_test::test;
use js_sys::{Function, JSON, JsString, JsValue, Reflect};

#[test]
fn parse_stringify_and_results() {
	let value = JSON::parse(r#"{"name":"js-bindgen"}"#).unwrap();
	assert_eq!(
		&Reflect::get_str(&value, "name").unwrap(),
		JsString::from("js-bindgen").as_ref()
	);
	assert_eq!(
		JSON::stringify(&value).unwrap().unwrap(),
		r#"{"name":"js-bindgen"}"#
	);
	assert!(JSON::stringify(&JsValue::UNDEFINED).unwrap().is_none());
	assert!(JSON::parse("{").is_err());
}

#[test]
fn callback_overload() {
	let reviver =
		Function::new_with_args("key, value", "return key === 'discard' ? undefined : value")
			.unwrap();
	let source = r#"{"keep":"yes","discard":"no"}"#;
	let value = JSON::parse_with_reviver(source, &reviver).unwrap();
	assert!(Reflect::has_str(&value, "keep").unwrap());
	assert!(!Reflect::has_str(&value, "discard").unwrap());
}

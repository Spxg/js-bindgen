use js_bindgen_test::test;
use js_sys::{JsString, JsValue, js_sys};

js_bindgen::embed_js!(module = "value", name = "nan", "() => NaN");
js_bindgen::embed_js!(
	module = "value",
	name = "externref_length",
	required_embeds = [("js_sys", "externref.table")],
	"() => this.#jsEmbed.js_sys['externref.table'].length",
);
js_bindgen::embed_js!(
	module = "value",
	name = "throwing_coercion",
	"() => ({{ [Symbol.toPrimitive]() {{ throw 'error' }} }})",
);

#[js_sys]
extern "js-sys" {
	#[js_sys(js_embed = "nan")]
	fn nan() -> JsValue;

	#[js_sys(js_embed = "throwing_coercion")]
	fn throwing_coercion() -> JsValue;

	#[js_sys(js_embed = "externref_length")]
	fn externref_length() -> u32;
}

#[test]
fn undefined() {
	let value = JsValue::UNDEFINED.clone();
	let string = JsString::new(&value).unwrap();
	let string = String::from(&string);

	assert_eq!(string, "undefined");
}

#[test]
fn null() {
	let value = JsValue::NULL.clone();
	let string = JsString::new(&value).unwrap();
	let string = String::from(&string);

	assert_eq!(string, "null");
}

#[test]
fn string_coercion_error() {
	assert!(JsString::new(&throwing_coercion()).is_err());
}

#[test]
fn clone() {
	let value = JsString::from("Hello, World!");
	let value = value.clone();
	assert_eq!(value, "Hello, World!");
}

#[test]
fn strict_equality_is_not_reflexive() {
	let value = nan();
	assert!(!PartialEq::eq(&value, &value));
}

#[test]
fn externref_slots_are_reused() {
	let value = JsString::from("Hello, World!");
	let values: Vec<_> = (0..512).map(|_| value.clone()).collect();
	assert!(values.iter().all(|candidate| candidate == &value));
	let grown_length = externref_length();
	drop(values);

	let reused: Vec<_> = (0..512).map(|_| value.clone()).collect();
	assert!(reused.iter().all(|candidate| candidate == &value));
	assert_eq!(externref_length(), grown_length);
}

#![expect(
	clippy::float_cmp,
	reason = "these decimal values are exactly representable"
)]

use js_bindgen_test::test;
use js_sys::{
	JsString, JsValue, Symbol, decode_uri, decode_uri_component, encode_uri, encode_uri_component,
	eval, global_this, is_finite, is_nan, parse_float, parse_int, parse_int_with_radix,
};

#[test]
fn uri_functions() {
	assert_eq!(
		encode_uri("https://example.com/a b?q=x/y"),
		"https://example.com/a%20b?q=x/y"
	);
	assert_eq!(encode_uri_component("a b?q=x/y"), "a%20b%3Fq%3Dx%2Fy");
	assert_eq!(
		decode_uri("https://example.com/a%20b?q=x/y").unwrap(),
		"https://example.com/a b?q=x/y"
	);
	assert_eq!(
		decode_uri_component("a%20b%3Fq%3Dx%2Fy").unwrap(),
		"a b?q=x/y"
	);
	assert!(decode_uri("%").is_err());
	assert!(decode_uri_component("%zz").is_err());
}

#[test]
fn numeric_functions() {
	assert!(is_finite(&eval("3").unwrap()).unwrap());
	assert!(!is_finite(&eval("Infinity").unwrap()).unwrap());
	assert!(is_nan(&JsValue::from(JsString::from("not a number"))).unwrap());
	assert!(is_finite(Symbol::new().as_ref()).is_err());
	assert!(is_nan(Symbol::new().as_ref()).is_err());

	assert_eq!(parse_float("3.25 trailing"), 3.25);
	assert_eq!(parse_int("17px"), 17.0);
	assert_eq!(parse_int_with_radix("ff", 16), 255.0);
}

#[test]
fn global_and_eval() {
	assert_ne!(global_this(), JsValue::NULL);
	assert_eq!(eval("1 + 2").unwrap(), eval("3").unwrap());
	assert!(eval("(").is_err());
}

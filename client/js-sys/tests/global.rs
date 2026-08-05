#![expect(
	clippy::float_cmp,
	reason = "these decimal values are exactly representable"
)]

use js_bindgen_test::test;
use js_sys::{
	JsValue, decode_uri_component, encode_uri_component, eval, global_this, parse_int_with_radix,
};

#[test]
fn global_functions() {
	assert_eq!(encode_uri_component("a b?q=x/y"), "a%20b%3Fq%3Dx%2Fy");
	assert_eq!(
		decode_uri_component("a%20b%3Fq%3Dx%2Fy").unwrap(),
		"a b?q=x/y"
	);
	assert!(decode_uri_component("%zz").is_err());
	assert_eq!(parse_int_with_radix("ff", 16), 255.0);
	assert_ne!(global_this(), JsValue::NULL);
	assert_eq!(eval("1 + 2").unwrap(), eval("3").unwrap());
	assert!(eval("(").is_err());
}

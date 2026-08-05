#![expect(
	clippy::float_cmp,
	reason = "RegExp lastIndex is tested with exact integral JavaScript Number values"
)]

use js_bindgen_test::test;
use js_sys::{JsString, RegExp};

#[test]
fn constructor_properties_and_result() {
	let regexp = RegExp::new_with_flags("a", "g").unwrap();
	assert_eq!(regexp.source(), JsString::from("a"));
	assert!(regexp.global());
	assert_eq!(
		regexp.exec("cat").unwrap().get(0),
		Some(JsString::from("a"))
	);

	regexp.set_last_index(2.0);
	assert_eq!(regexp.last_index(), 2.0);
	assert!(RegExp::new("[").is_err());
}

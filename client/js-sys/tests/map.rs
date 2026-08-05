#![expect(
	clippy::float_cmp,
	reason = "map sizes are exact integral JavaScript Number values"
)]

use js_bindgen_test::test;
use js_sys::{JsString, Map};

#[test]
fn core_api() {
	let first_key = JsString::from("first");
	let first_value = JsString::from("one");
	let map = Map::<JsString, JsString>::new_typed();

	let _ = map.set(&first_key, &first_value);
	assert_eq!(map.size(), 1.0);
	assert_eq!(map.get_checked(&first_key), Some(first_value.clone()));

	let entries = (&map).into_iter().collect::<Result<Vec<_>, _>>().unwrap();
	assert_eq!(entries.len(), 1);

	assert!(map.delete(&first_key));
	assert_eq!(map.size(), 0.0);
}

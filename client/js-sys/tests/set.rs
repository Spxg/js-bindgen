#![expect(
	clippy::float_cmp,
	reason = "set sizes are exact integral JavaScript Number values"
)]

use js_bindgen_test::test;
use js_sys::{JsString, Set};

#[test]
fn core_api() {
	let first = JsString::from("first");
	let set = Set::<JsString>::new_typed();

	assert_eq!(set.size(), 0.0);
	let _ = set.add(&first);
	assert_eq!(set.size(), 1.0);
	assert!(set.has(&first));

	let iterated: Vec<_> = (&set).into_iter().collect::<Result<_, _>>().unwrap();
	assert_eq!(iterated.as_slice(), core::slice::from_ref(&first));

	assert!(set.delete(&first));
	assert_eq!(set.size(), 0.0);
}

use js_bindgen_test::test;
use js_sys::{JsString, Object, WeakSet};

#[test]
fn constructor_and_instance_methods() {
	let value = Object::new();
	let set = WeakSet::<Object>::new_typed();

	let _ = set.add(&value).unwrap();
	assert!(set.has(&value));
	assert!(set.delete(&value));
	assert!(
		WeakSet::<JsString>::new_typed()
			.add(&JsString::from("not an object"))
			.is_err()
	);
}

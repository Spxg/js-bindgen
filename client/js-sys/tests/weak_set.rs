use js_bindgen_test::test;
use js_sys::{Array, JsString, Object, WeakSet};

#[test]
fn core_api() {
	let value = Object::new();
	let missing = Object::new();
	let set = WeakSet::<Object>::new_typed();

	assert!(!set.has(&value));
	let _ = set.add(&value).unwrap();
	assert!(set.has(&value));
	assert!(!set.has(&missing));
	assert!(set.delete(&value));
	assert!(!set.delete(&value));
}

#[test]
fn iterable_constructor() {
	let value = Object::new();
	let values = Array::<Object>::new_typed();
	let _ = values.push(&value);

	let set = WeakSet::<Object>::new_from_iterable(&values).unwrap();
	assert!(set.has(&value));
	assert!(!WeakSet::<Object>::default().has(&value));
}

#[test]
fn add_reports_invalid_weak_values() {
	let set = WeakSet::<JsString>::new_typed();
	assert!(set.add(&JsString::from("not an object")).is_err());
}

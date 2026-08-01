use js_bindgen_test::test;
use js_sys::{Array, Function, JsString, JsValue, Object, WeakMap};

fn entry(key: &Object, value: &JsString) -> Array {
	let entry = Array::new();
	let _ = entry.push(key.as_ref());
	let _ = entry.push(value.as_ref());
	entry
}

#[test]
fn core_api() {
	let key = Object::new();
	let missing = Object::new();
	let value = JsString::from("value");
	let map = WeakMap::<Object, JsString>::new_typed();

	assert!(!map.has(&key));
	assert!(map.get_checked(&key).is_none());
	let _ = map.set(&key, &value).unwrap();
	assert!(map.has(&key));
	assert_eq!(map.get_checked(&key), Some(value.clone()));
	assert!(map.get_checked(&missing).is_none());
	assert!(map.delete(&key));
	assert!(!map.delete(&key));
}

#[test]
fn iterable_constructor() {
	let key = Object::new();
	let value = JsString::from("value");
	let entries = Array::<Array>::new_typed();
	let _ = entries.push(&entry(&key, &value));

	let map = WeakMap::<Object, JsString>::new_from_iterable(&entries).unwrap();
	assert_eq!(map.get_checked(&key), Some(value));
	assert!(!WeakMap::<Object, JsString>::default().has(&key));
}

#[test]
fn get_preserves_null_and_undefined() {
	let map = WeakMap::new();
	let present = Object::new();
	let missing = Object::new();
	let _ = map.set(&present, &JsValue::NULL).unwrap();

	assert_eq!(map.get(&present), JsValue::NULL);
	assert_eq!(map.get(&missing), JsValue::UNDEFINED);
}

#[test]
fn set_reports_invalid_weak_keys() {
	let map = WeakMap::<JsString, JsValue>::new_typed();
	assert!(
		map.set(&JsString::from("not an object"), &JsValue::NULL)
			.is_err()
	);
}

#[test]
fn get_or_insert() {
	let map = WeakMap::<Object, JsString>::new_typed();
	let present = Object::new();
	let missing = Object::new();
	let computed = Object::new();
	let original = JsString::from("original");
	let fallback = JsString::from("fallback");
	let _ = map.set(&present, &original).unwrap();

	assert_eq!(map.get_or_insert(&present, &fallback).unwrap(), original);
	assert_eq!(map.get_or_insert(&missing, &fallback).unwrap(), fallback);

	let callback = Function::new_no_args("return 'computed'").unwrap();
	assert_eq!(
		map.get_or_insert_computed(&computed, &callback).unwrap(),
		JsString::from("computed")
	);
	assert_eq!(
		map.get_or_insert_computed(&computed, &Function::new_no_args("throw null").unwrap())
			.unwrap(),
		JsString::from("computed")
	);
	let missing = Object::new();
	let throwing = Function::new_no_args("throw null").unwrap();
	assert!(map.get_or_insert_computed(&missing, &throwing).is_err());

	let invalid = WeakMap::<JsString, JsString>::new_typed();
	assert!(
		invalid
			.get_or_insert(&JsString::from("key"), &fallback)
			.is_err()
	);
}

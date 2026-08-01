#![expect(
	clippy::float_cmp,
	reason = "map sizes are exact integral JavaScript Number values"
)]

use js_bindgen_test::test;
use js_sys::{Array, Function, JsString, JsValue, Map};

fn entry(key: &JsString, value: &JsString) -> Array {
	let entry = Array::new();
	let _ = entry.push(key.as_ref());
	let _ = entry.push(value.as_ref());
	entry
}

#[test]
fn core_api() {
	let first_key = JsString::from("first");
	let second_key = JsString::from("second");
	let first_value = JsString::from("one");
	let second_value = JsString::from("two");
	let map = Map::<JsString, JsString>::new_typed();

	assert_eq!(map.size(), 0.0);
	assert!(!map.has(&first_key));
	assert!(map.get_checked(&first_key).is_none());

	let _ = map.set(&first_key, &first_value);
	let _ = map.set(&second_key, &second_value);
	assert_eq!(map.size(), 2.0);
	assert_eq!(map.get_checked(&first_key), Some(first_value.clone()));
	assert!(map.has(&second_key));

	let keys: Vec<_> = map.keys().into_iter().collect::<Result<_, _>>().unwrap();
	assert_eq!(keys, [first_key.clone(), second_key.clone()]);
	let values: Vec<_> = map.values().into_iter().collect::<Result<_, _>>().unwrap();
	assert_eq!(values, [first_value.clone(), second_value.clone()]);

	let entries: Vec<_> = (&map).into_iter().collect::<Result<_, _>>().unwrap();
	assert_eq!(entries.len(), 2);
	assert_eq!(entries[0].get(0), JsValue::from(first_key.clone()));
	assert_eq!(entries[0].get(1), JsValue::from(first_value.clone()));
	assert_eq!(entries[1].get(0), JsValue::from(second_key.clone()));
	assert_eq!(entries[1].get(1), JsValue::from(second_value.clone()));

	let throwing = Function::new_no_args("throw new Error('boom')").unwrap();
	assert!(map.for_each(&throwing).is_err());

	assert!(map.delete(&first_key));
	assert!(!map.delete(&first_key));
	map.clear();
	assert_eq!(map.size(), 0.0);
}

#[test]
fn iterable_constructor() {
	let first_key = JsString::from("first");
	let second_key = JsString::from("second");
	let first_value = JsString::from("one");
	let second_value = JsString::from("two");
	let entries = Array::<Array>::new_typed();
	let _ = entries.push(&entry(&first_key, &first_value));
	let _ = entries.push(&entry(&second_key, &second_value));

	let map = Map::<JsString, JsString>::new_from_iterable(&entries).unwrap();
	assert_eq!(map.size(), 2.0);
	assert_eq!(map.get_checked(&first_key), Some(first_value));
	assert_eq!(map.get_checked(&second_key), Some(second_value));

	let empty = Map::<JsString, JsString>::default();
	assert_eq!(empty.size(), 0.0);
}

#[test]
fn get_preserves_null_and_undefined() {
	let map = Map::new();
	let present = JsValue::from(JsString::from("present"));
	let missing = JsValue::from(JsString::from("missing"));
	let _ = map.set(&present, &JsValue::NULL);

	assert_eq!(map.get(&present), JsValue::NULL);
	assert_eq!(map.get(&missing), JsValue::UNDEFINED);
}

#[test]
fn get_or_insert() {
	let map = Map::<JsString, JsString>::new_typed();
	let present = JsString::from("present");
	let missing = JsString::from("missing");
	let computed = JsString::from("computed");
	let original = JsString::from("original");
	let fallback = JsString::from("fallback");
	let _ = map.set(&present, &original);

	assert_eq!(map.get_or_insert(&present, &fallback), original);
	assert_eq!(map.get_or_insert(&missing, &fallback), fallback);

	let callback = Function::new_with_args("key", "return `${key}:computed`").unwrap();
	assert_eq!(
		map.get_or_insert_computed(&computed, &callback).unwrap(),
		JsString::from("computed:computed")
	);
	assert_eq!(
		map.get_or_insert_computed(&computed, &Function::new_no_args("throw null").unwrap())
			.unwrap(),
		JsString::from("computed:computed")
	);

	let throwing = Function::new_no_args("throw null").unwrap();
	assert!(
		map.get_or_insert_computed(&JsString::from("throws"), &throwing)
			.is_err()
	);
}

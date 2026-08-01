#![expect(
	clippy::float_cmp,
	reason = "set sizes are exact integral JavaScript Number values"
)]

use js_bindgen_test::test;
use js_sys::{Array, Function, JsString, JsValue, Set};

fn values(values: &[&str]) -> Array<JsString> {
	let array = Array::new_typed();
	for value in values {
		let _ = array.push(&JsString::from(*value));
	}
	array
}

#[test]
fn core_api() {
	let first = JsString::from("first");
	let second = JsString::from("second");
	let set = Set::<JsString>::new_typed();

	assert_eq!(set.size(), 0.0);
	let _ = set.add(&first);
	let _ = set.add(&second);
	let _ = set.add(&first);
	assert_eq!(set.size(), 2.0);
	assert!(set.has(&first));

	let iterated: Vec<_> = (&set).into_iter().collect::<Result<_, _>>().unwrap();
	assert_eq!(iterated, [first.clone(), second.clone()]);
	let keys: Vec<_> = set.keys().into_iter().collect::<Result<_, _>>().unwrap();
	let values: Vec<_> = set.values().into_iter().collect::<Result<_, _>>().unwrap();
	assert_eq!(keys, values);
	let entries: Vec<_> = set.entries().into_iter().collect::<Result<_, _>>().unwrap();
	assert_eq!(entries[0].get(0), JsValue::from(first.clone()));
	assert_eq!(entries[0].get(1), JsValue::from(first.clone()));
	assert_eq!(entries[1].get(0), JsValue::from(second.clone()));
	assert_eq!(entries[1].get(1), JsValue::from(second.clone()));

	let throwing = Function::new_no_args("throw new Error('boom')").unwrap();
	assert!(set.for_each(&throwing).is_err());

	assert!(set.delete(&first));
	assert!(!set.delete(&first));
	set.clear();
	assert_eq!(set.size(), 0.0);
}

#[test]
fn iterable_constructor_and_composition() {
	let first = JsString::from("first");
	let second = JsString::from("second");
	let third = JsString::from("third");
	let left = Set::<JsString>::new_from_iterable(&values(&["first", "second"])).unwrap();
	let right = Set::<JsString>::new_from_iterable(&values(&["second", "third"])).unwrap();

	let union = left.union(&right);
	assert_eq!(union.size(), 3.0);
	assert!(union.has(&first));
	assert!(union.has(&second));
	assert!(union.has(&third));

	let intersection = left.intersection(&right);
	assert_eq!(intersection.size(), 1.0);
	assert!(intersection.has(&second));

	let difference = left.difference(&right);
	assert_eq!(difference.size(), 1.0);
	assert!(difference.has(&first));

	let symmetric = left.symmetric_difference(&right);
	assert_eq!(symmetric.size(), 2.0);
	assert!(symmetric.has(&first));
	assert!(symmetric.has(&third));

	assert!(intersection.is_subset_of(&left));
	assert!(union.is_superset_of(&right));
	assert!(difference.is_disjoint_from(&right));
	assert_eq!(Set::<JsString>::default().size(), 0.0);
}

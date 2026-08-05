use js_bindgen_test::test;
use js_sys::{JsString, JsValue, Object, WeakMap};

#[test]
fn core_api() {
	let key = Object::new();
	let value = JsString::from("value");
	let map = WeakMap::<Object, JsString>::new_typed();

	let _ = map.set(&key, &value).unwrap();
	assert!(map.has(&key));
	assert_eq!(map.get_checked(&key), Some(value));
	assert!(map.delete(&key));
	let map = WeakMap::<JsString, JsValue>::new_typed();
	assert!(
		map.set(&JsString::from("not an object"), &JsValue::NULL)
			.is_err()
	);
}

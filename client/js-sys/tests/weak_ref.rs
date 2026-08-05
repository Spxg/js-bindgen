use js_bindgen_test::test;
use js_sys::{JsString, Object, WeakRef};

#[test]
fn constructor_and_deref() {
	let target = Object::new();
	let weak = WeakRef::new(&target).unwrap();
	assert_eq!(weak.deref().unwrap().as_ref(), target.as_ref());
	assert!(WeakRef::new(&JsString::from("primitive")).is_err());
}

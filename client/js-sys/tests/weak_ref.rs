use js_bindgen_test::test;
use js_sys::{JsString, Object, Symbol, WeakRef};

#[test]
fn object_target() {
	let target = Object::new();
	let weak = WeakRef::new(&target).unwrap();
	let value = weak.deref().unwrap();

	assert_eq!(value.as_ref(), target.as_ref());
}

#[test]
fn non_registered_symbol_target() {
	let target = Symbol::new_with_description("target");
	let weak = WeakRef::new(&target).unwrap();

	assert_eq!(weak.deref(), Some(target));
}

#[test]
fn invalid_targets_are_reported() {
	assert!(WeakRef::new(&JsString::from("primitive")).is_err());
	assert!(WeakRef::new(&Symbol::for_("registered")).is_err());
}

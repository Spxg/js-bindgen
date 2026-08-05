use js_bindgen_test::test;
use js_sys::{FinalizationRegistry, Function, JsString, Object};

fn registry() -> FinalizationRegistry {
	let cleanup = Function::new_no_args("").unwrap();
	FinalizationRegistry::new(&cleanup).unwrap()
}

#[test]
fn register_and_unregister() {
	let registry = registry();
	let target = Object::new();
	let token = Object::new();
	let held = JsString::from("held");

	registry
		.register_with_token(target.as_ref(), held.as_ref(), token.as_ref())
		.unwrap();
	assert!(registry.unregister(token.as_ref()).unwrap());
}

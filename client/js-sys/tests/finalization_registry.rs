use js_bindgen_test::test;
use js_sys::hazard::JsCast;
use js_sys::{FinalizationRegistry, Function, JsString, Object, Symbol};

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
	assert!(!registry.unregister(token.as_ref()).unwrap());
}

#[test]
fn accepts_weakly_holdable_symbols() {
	let registry = registry();
	let target = Symbol::new_with_description("target");
	let token = Symbol::new_with_description("token");
	let held = JsString::from("held");

	registry
		.register_with_token(target.as_ref(), held.as_ref(), token.as_ref())
		.unwrap();
	assert!(registry.unregister(token.as_ref()).unwrap());
}

#[test]
fn reports_invalid_registration_values() {
	let registry = registry();
	let target = Object::new();
	let primitive = JsString::from("primitive");
	let registered_symbol = Symbol::for_("registered");

	assert!(
		registry
			.register(primitive.as_ref(), target.as_ref())
			.is_err()
	);
	assert!(registry.register(target.as_ref(), target.as_ref()).is_err());
	assert!(
		registry
			.register(registered_symbol.as_ref(), target.as_ref())
			.is_err()
	);
	assert!(
		registry
			.register_with_token(target.as_ref(), primitive.as_ref(), primitive.as_ref())
			.is_err()
	);
	assert!(registry.unregister(primitive.as_ref()).is_err());
}

#[test]
fn reports_invalid_cleanup_callback() {
	let value = JsString::from("not callable");
	let callback = Function::unchecked_from(value.into());

	assert!(FinalizationRegistry::new(&callback).is_err());
}

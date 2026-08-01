use js_bindgen_test::test;
use js_sys::hazard::JsCast;
use js_sys::{Boolean, DisposableStack, Function, JsValue, Object, Reflect};

fn property_is_true(object: &Object, name: &str) -> bool {
	let value = Reflect::get_str(object.as_ref(), name).unwrap();
	Boolean::new_with_value(&value).value_of()
}

#[test]
fn adopt_and_move() {
	let stack = DisposableStack::new();
	let value = Object::new();
	let callback = Function::new_with_args("value", "value.disposed = true").unwrap();
	let returned: Object = stack.adopt(&value, &callback).unwrap();

	assert_eq!(returned.as_ref(), value.as_ref());
	assert!(!stack.disposed());

	let moved = stack.move_().unwrap();
	assert!(stack.disposed());
	assert!(!moved.disposed());
	moved.dispose().unwrap();
	assert!(moved.disposed());
	assert!(property_is_true(&value, "disposed"));
}

#[test]
fn use_and_symbol_dispose() {
	let make_resource = Function::new_no_args(
		"return { disposed: false, [Symbol.dispose]() { this.disposed = true; } }",
	)
	.unwrap();
	let resource = Object::unchecked_from(make_resource.call(&JsValue::UNDEFINED, &[]).unwrap());
	let stack = DisposableStack::default();
	let returned: Object = stack.use_(&resource).unwrap();

	assert_eq!(returned.as_ref(), resource.as_ref());
	stack.symbol_dispose().unwrap();
	assert!(stack.disposed());
	assert!(property_is_true(&resource, "disposed"));
}

#[test]
fn exceptions_are_returned() {
	let stack = DisposableStack::new();
	let throwing = Function::new_no_args("throw new Error('dispose failed')").unwrap();
	stack.defer(&throwing).unwrap();

	assert!(stack.dispose().is_err());
	assert!(stack.disposed());
	assert!(stack.defer(&throwing).is_err());
}

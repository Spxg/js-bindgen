use js_bindgen_test::test;
use js_sys::hazard::JsCast;
use js_sys::{AsyncDisposableStack, Boolean, Function, JsValue, Object, Reflect};

fn property_is_true(object: &Object, name: &str) -> bool {
	let value = Reflect::get_str(object.as_ref(), name).unwrap();
	Boolean::new_with_value(&value).value_of()
}

#[test]
async fn adopt_and_move() {
	let stack = AsyncDisposableStack::new();
	let value = Object::new();
	let callback = Function::new_with_args(
		"value",
		"return Promise.resolve().then(() => { value.disposed = true; })",
	)
	.unwrap();
	let returned: Object = stack.adopt(&value, &callback).unwrap();

	assert_eq!(returned.as_ref(), value.as_ref());
	let moved = stack.move_().unwrap();
	assert!(stack.disposed());
	assert!(!moved.disposed());
	moved.dispose_async().await.unwrap();
	assert!(moved.disposed());
	assert!(property_is_true(&value, "disposed"));
}

#[test]
async fn use_and_symbol_async_dispose() {
	let make_resource = Function::new_no_args(
		"return { disposed: false, [Symbol.asyncDispose]() { this.disposed = true; return \
		 Promise.resolve(); } }",
	)
	.unwrap();
	let resource = Object::unchecked_from(make_resource.call(&JsValue::UNDEFINED, &[]).unwrap());
	let stack = AsyncDisposableStack::default();
	let returned: Object = stack.use_(&resource).unwrap();

	assert_eq!(returned.as_ref(), resource.as_ref());
	stack.symbol_async_dispose().await.unwrap();
	assert!(stack.disposed());
	assert!(property_is_true(&resource, "disposed"));
}

#[test]
async fn rejected_disposal_is_returned() {
	let stack = AsyncDisposableStack::new();
	let rejecting =
		Function::new_no_args("return Promise.reject(new Error('dispose failed'))").unwrap();
	stack.defer(&rejecting).unwrap();

	assert!(stack.dispose_async().await.is_err());
	assert!(stack.disposed());
	assert!(stack.defer(&rejecting).is_err());
}

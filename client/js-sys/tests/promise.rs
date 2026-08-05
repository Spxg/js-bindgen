use js_bindgen_test::test;
use js_sys::hazard::JsCast;
use js_sys::{Array, Function, JsString, JsValue, Promise, closure};

fn string(value: &str) -> JsValue {
	JsString::from(value).into()
}

#[test]
async fn constructor_resolves() {
	let expected = string("constructed");
	let value = expected.clone();
	let executor = closure!(dyn FnMut(Function, Function), move |resolve, _reject| {
		resolve
			.call(&JsValue::UNDEFINED, core::slice::from_ref(&value))
			.unwrap();
	});

	assert_eq!(Promise::new(&executor).await.unwrap(), expected);
}

#[test]
async fn static_iterable_and_variadic_bindings() {
	let values = Array::of(&[string("first"), string("second")]);
	let all: Array = Array::unchecked_from(Promise::all(values.as_ref()).await.unwrap());
	assert_eq!(all.get(0), string("first"));
	assert_eq!(all.get(1), string("second"));

	let concatenate = Function::new_with_args("left, right", "return `${left}:${right}`").unwrap();
	let result = Promise::try_(&concatenate, &[string("one"), string("two")])
		.await
		.unwrap();
	assert_eq!(result, string("one:two"));

	let error = string("rejected");
	assert_eq!(Promise::reject(&error).await.unwrap_err(), error);
}

#[test]
async fn with_resolvers() {
	let resolvers = Promise::with_resolvers();
	let promise = resolvers.promise();
	let expected = string("resolved");
	resolvers
		.resolve()
		.call(&JsValue::UNDEFINED, core::slice::from_ref(&expected))
		.unwrap();
	assert_eq!(promise.await.unwrap(), expected);
}

#[test]
async fn reaction_bindings() {
	let then = closure!(dyn FnMut(JsValue) -> JsValue, |value| {
		assert_eq!(value, string("input"));
		string("mapped")
	});
	let mapped = Promise::resolve(&string("input")).then(then).await.unwrap();
	assert_eq!(mapped, string("mapped"));

	let resolve = closure!(dyn FnMut(JsValue) -> JsValue, |_| string("unexpected"));
	let reject = closure!(dyn FnMut(JsValue) -> JsValue, |error| {
		assert_eq!(error, string("failure"));
		string("handled")
	});
	assert_eq!(
		Promise::reject(&string("failure"))
			.then_with_reject(resolve, reject)
			.await
			.unwrap(),
		string("handled")
	);
}

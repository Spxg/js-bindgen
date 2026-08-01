use core::cell::Cell;
use std::rc::Rc;

use js_bindgen_test::test;
use js_sys::hazard::JsCast;
use js_sys::{Array, Function, JsString, JsValue, Promise, Reflect, closure};

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
async fn static_combinators() {
	let values = Array::of(&[string("first"), string("second")]);
	let all: Array = Array::unchecked_from(Promise::all(values.as_ref()).await.unwrap());
	assert_eq!(all.get(0), string("first"));
	assert_eq!(all.get(1), string("second"));

	let candidates = Array::new();
	assert_eq!(
		candidates.push(Promise::resolve(&string("value")).as_ref()),
		1
	);
	assert_eq!(
		candidates.push(Promise::reject(&string("reason")).as_ref()),
		2
	);
	let settled: Array =
		Array::unchecked_from(Promise::all_settled(candidates.as_ref()).await.unwrap());
	assert_eq!(
		Reflect::get_str(&settled.get(0), "status").unwrap(),
		string("fulfilled")
	);
	assert_eq!(
		Reflect::get_str(&settled.get(0), "value").unwrap(),
		string("value")
	);
	assert_eq!(
		Reflect::get_str(&settled.get(1), "status").unwrap(),
		string("rejected")
	);
	assert_eq!(
		Reflect::get_str(&settled.get(1), "reason").unwrap(),
		string("reason")
	);

	let rejected = Promise::reject(&string("ignored"));
	let fulfilled = Promise::resolve(&string("accepted"));
	let candidates = Array::new();
	assert_eq!(candidates.push(rejected.as_ref()), 1);
	assert_eq!(candidates.push(fulfilled.as_ref()), 2);
	assert_eq!(
		Promise::any(candidates.as_ref()).await.unwrap(),
		string("accepted")
	);

	let values = Array::of(&[string("winner"), string("later")]);
	assert_eq!(
		Promise::race(values.as_ref()).await.unwrap(),
		string("winner")
	);
}

#[test]
async fn reject_and_try() {
	let error = string("rejected");
	assert_eq!(Promise::reject(&error).await.unwrap_err(), error);

	let concatenate = Function::new_with_args("left, right", "return `${left}:${right}`").unwrap();
	let result = Promise::try_(&concatenate, &[string("one"), string("two")])
		.await
		.unwrap();
	assert_eq!(result, string("one:two"));
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

	let resolvers = Promise::with_resolvers();
	let promise = resolvers.promise();
	let expected = string("rejected");
	resolvers
		.reject()
		.call(&JsValue::UNDEFINED, core::slice::from_ref(&expected))
		.unwrap();
	assert_eq!(promise.await.unwrap_err(), expected);
}

#[test]
async fn reactions() {
	let then = closure!(dyn FnMut(JsValue) -> JsValue, |value| {
		assert_eq!(value, string("input"));
		string("mapped")
	});
	let mapped = Promise::resolve(&string("input")).then(then).await.unwrap();
	assert_eq!(mapped, string("mapped"));

	let catch = closure!(dyn FnMut(JsValue) -> JsValue, |error| {
		assert_eq!(error, string("failure"));
		string("recovered")
	});
	let recovered = Promise::reject(&string("failure"))
		.catch(catch)
		.await
		.unwrap();
	assert_eq!(recovered, string("recovered"));

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

	let called = Rc::new(Cell::new(false));
	let callback_called = Rc::clone(&called);
	let finally = closure!(dyn FnMut(), move || callback_called.set(true));
	let value = Promise::resolve(&string("preserved"))
		.finally(finally)
		.await
		.unwrap();
	assert_eq!(value, string("preserved"));
	assert!(called.get());
}

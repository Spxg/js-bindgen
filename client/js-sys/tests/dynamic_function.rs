use js_bindgen_test::test;
use js_sys::{
	Array, AsyncFunction, AsyncGeneratorFunction, AsyncIterator, Function, GeneratorFunction,
	JsFuture, JsIterator, JsString, JsValue, Object, Reflect,
};

fn assert_function<T: AsRef<Function> + AsRef<Object>>() {}

#[test]
fn function_methods() {
	let function =
		Function::new_with_args("left, right", "return `${this.prefix}:${left}:${right}`").unwrap();
	let this = Object::new();
	let prefix = JsString::from("prefix");
	assert!(Reflect::set_str(this.as_ref(), "prefix", prefix.as_ref()).unwrap());
	let left = JsValue::from(JsString::from("left"));
	let right = JsValue::from(JsString::from("right"));

	let bound = function.bind(this.as_ref(), core::slice::from_ref(&left));
	assert_eq!(
		bound
			.call(&JsValue::UNDEFINED, core::slice::from_ref(&right))
			.unwrap(),
		JsValue::from(JsString::from("prefix:left:right"))
	);

	let arguments = Array::of(&[left, right]);
	assert_eq!(
		function.apply(this.as_ref(), arguments.as_ref()).unwrap(),
		JsValue::from(JsString::from("prefix:left:right"))
	);

	let prototype = Object::new();
	function.set_prototype(prototype.as_ref());
	assert_eq!(&function.prototype(), prototype.as_ref());
}

#[test]
fn generator_function_constructs_and_calls() {
	assert_function::<GeneratorFunction>();

	let function = GeneratorFunction::<JsString, JsString, JsString>::new_with_args(
		"value",
		"yield value; return 'done'",
	)
	.unwrap();
	let args = [JsValue::from(JsString::from("yielded"))];
	let generator = function.call(&JsValue::UNDEFINED, &args).unwrap();
	let iterator: &JsIterator<JsString> = generator.as_ref();

	let yielded = iterator.next_result().unwrap();
	assert!(!yielded.done().unwrap());
	assert_eq!(yielded.value().unwrap(), JsString::from("yielded").into());
	assert!(iterator.next_result().unwrap().done().unwrap());
}

#[test]
async fn asynchronous_function_variants_construct_and_call() {
	assert_function::<AsyncFunction>();
	assert_function::<AsyncGeneratorFunction>();

	let function =
		AsyncFunction::<JsString>::new_with_args("value", "return value.toUpperCase()").unwrap();
	let args = [JsValue::from(JsString::from("async"))];
	let value = JsFuture::from(function.call(&JsValue::UNDEFINED, &args).unwrap())
		.await
		.unwrap();
	assert_eq!(value, "ASYNC");

	let function = AsyncGeneratorFunction::<JsString, JsString, JsString>::new_with_args(
		"value",
		"yield await Promise.resolve(value); return 'done'",
	)
	.unwrap();
	let generator = function.call(&JsValue::UNDEFINED, &args).unwrap();
	let iterator: AsyncIterator<JsString> = generator.into();
	let mut iterator = iterator.into_async_iter();
	assert_eq!(iterator.next().await.unwrap().unwrap(), "async");
	assert!(iterator.next().await.is_none());

	let throwing = AsyncFunction::<JsValue>::new("throw new Error('async')").unwrap();
	let promise = throwing.call(&JsValue::UNDEFINED, &[]).unwrap();
	assert!(JsFuture::from(promise).await.is_err());
}

#[test]
fn invalid_bodies_are_reported() {
	assert!(GeneratorFunction::<JsValue, JsValue, JsValue>::new("yield (").is_err());
	assert!(AsyncGeneratorFunction::<JsValue, JsValue, JsValue>::new("yield (").is_err());
	assert!(AsyncFunction::<JsValue>::new("return (").is_err());
}

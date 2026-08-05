use js_bindgen_test::test;
use js_sys::{JsString, JsValue, Object, Proxy, Reflect};

#[test]
fn constructor_static_method_and_getters() {
	let target = Object::new();
	let handler = Object::new();
	let value = JsString::from("value");
	assert!(Reflect::set_str(target.as_ref(), "key", value.as_ref()).unwrap());

	let revocable = Proxy::revocable(target.as_ref(), handler.as_ref()).unwrap();
	let proxy = revocable.proxy();
	assert_eq!(
		&Reflect::get_str(proxy.as_ref(), "key").unwrap(),
		value.as_ref()
	);

	assert_eq!(
		revocable.revoke().call(&JsValue::UNDEFINED, &[]).unwrap(),
		JsValue::UNDEFINED
	);
	assert!(Reflect::get_str(proxy.as_ref(), "key").is_err());
	assert!(Proxy::new(&JsValue::NULL, handler.as_ref()).is_err());
}

use js_bindgen_test::test;
use js_sys::{JsString, JsValue, Object, Proxy, Reflect};

#[test]
fn forwards_to_the_target() {
	let target = Object::new();
	let handler = Object::new();
	let value = JsString::from("value");
	assert!(Reflect::set_str(target.as_ref(), "key", value.as_ref()).unwrap());

	let proxy = Proxy::new(target.as_ref(), handler.as_ref()).unwrap();
	assert_eq!(
		&Reflect::get_str(proxy.as_ref(), "key").unwrap(),
		value.as_ref()
	);
}

#[test]
fn revocable_proxy_stops_forwarding() {
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
}

#[test]
fn constructor_errors_are_returned() {
	let object = Object::new();

	assert!(Proxy::new(&JsValue::NULL, object.as_ref()).is_err());
	assert!(Proxy::new(object.as_ref(), &JsValue::NULL).is_err());
	assert!(Proxy::revocable(&JsValue::NULL, object.as_ref()).is_err());
	assert!(Proxy::revocable(object.as_ref(), &JsValue::NULL).is_err());
}

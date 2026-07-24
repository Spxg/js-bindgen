use core::{array, hint::black_box};

use js_sys::{JsValue, js_sys};

js_sys::js_bindgen::embed_js!(
	module = "js_bindgen_benchmark",
	name = "identity",
	"(value) => value",
);

#[js_sys]
extern "js-sys" {
	#[js_sys(js_embed = "identity")]
	fn import_i32_identity_raw(value: i32) -> i32;

	#[js_sys(js_embed = "identity")]
	fn import_u128_identity_raw(value: u128) -> u128;

	#[js_sys(js_embed = "identity")]
	fn import_option_i32_identity_raw(value: Option<i32>) -> Option<i32>;

	#[js_sys(js_embed = "identity")]
	fn import_result_i32_identity_raw(value: i32) -> Result<i32, JsValue>;

	#[js_sys(js_embed = "identity")]
	fn import_js_value_identity_raw(value: JsValue) -> JsValue;

	#[js_sys(js_embed = "identity")]
	fn import_js_value_identity_2_raw(value: JsValue, other: JsValue) -> JsValue;

	#[js_sys(js_embed = "identity")]
	fn import_js_value_identity_4_raw(
		value: JsValue,
		other1: JsValue,
		other2: JsValue,
		other3: JsValue,
	) -> JsValue;
}

#[js_sys]
fn bench_i32_identity(value: i32) -> i32 {
	value
}

#[js_sys]
fn bench_u128_identity(value: u128) -> u128 {
	value
}

#[js_sys]
fn bench_option_i32_some(value: i32) -> Option<i32> {
	Some(value)
}

#[js_sys]
fn bench_result_i32_ok(value: i32) -> Result<i32, JsValue> {
	Ok(value)
}

#[js_sys]
fn bench_js_value_identity(value: JsValue) -> JsValue {
	value
}

#[js_sys]
fn bench_js_value_create_512(value: JsValue) -> i32 {
	let values: [JsValue; 512] = array::from_fn(|_| value.clone());
	black_box(&values);
	512
}

#[js_sys]
fn bench_js_value_identity_2(value: JsValue, _other: JsValue) -> JsValue {
	value
}

#[js_sys]
fn bench_js_value_identity_4(
	value: JsValue,
	_other1: JsValue,
	_other2: JsValue,
	_other3: JsValue,
) -> JsValue {
	value
}

#[js_sys]
fn bench_import_i32_identity(value: i32) -> i32 {
	import_i32_identity_raw(value)
}

#[js_sys]
fn bench_import_u128_identity(value: u128) -> u128 {
	import_u128_identity_raw(value)
}

#[js_sys]
fn bench_import_option_i32_some(value: Option<i32>) -> Option<i32> {
	import_option_i32_identity_raw(value)
}

#[js_sys]
fn bench_import_result_i32_ok(value: i32) -> Result<i32, JsValue> {
	import_result_i32_identity_raw(value)
}

#[js_sys]
fn bench_import_js_value_identity(value: JsValue) -> JsValue {
	import_js_value_identity_raw(value)
}

#[js_sys]
fn bench_import_js_value_identity_2(value: JsValue, other: JsValue) -> JsValue {
	import_js_value_identity_2_raw(value, other)
}

#[js_sys]
fn bench_import_js_value_identity_4(
	value: JsValue,
	other1: JsValue,
	other2: JsValue,
	other3: JsValue,
) -> JsValue {
	import_js_value_identity_4_raw(value, other1, other2, other3)
}

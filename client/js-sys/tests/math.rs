use js_bindgen_test::test;
use js_sys::{Array, Math, js_sys};

js_bindgen::embed_js!(
	module = "math",
	name = "sum_precise.supported",
	"() => typeof Math.sumPrecise === 'function'",
);
js_bindgen::embed_js!(
	module = "math",
	name = "sum_precise.numbers",
	"() => [1e20, 0.1, -1e20]",
);

#[js_sys]
extern "js-sys" {
	#[js_sys(js_embed = "sum_precise.supported")]
	fn sum_precise_supported() -> bool;

	#[js_sys(js_embed = "sum_precise.numbers")]
	fn precise_numbers() -> Array;
}

fn assert_close(actual: f64, expected: f64) {
	let tolerance = f64::EPSILON * expected.abs().max(1.0);
	assert!(
		(actual - expected).abs() <= tolerance,
		"expected {expected}, got {actual}"
	);
}

#[test]
fn constants() {
	assert_close(Math::e(), core::f64::consts::E);
	assert_close(Math::pi(), core::f64::consts::PI);
	assert_close(Math::sqrt_2(), core::f64::consts::SQRT_2);
}

#[test]
fn arithmetic() {
	assert_close(Math::abs(-2.0), 2.0);
	assert_close(Math::ceil(1.25), 2.0);
	assert_eq!(Math::clz32(1), 31);
	assert_close(Math::floor(1.75), 1.0);
	assert_eq!(Math::imul(0x7fff_ffff, 2), -2);
	assert_close(Math::pow(2.0, 10.0), 1024.0);
	assert_close(Math::round(1.5), 2.0);
	assert_close(Math::trunc(-1.75), -1.0);
}

#[test]
fn variadic() {
	assert_close(Math::hypot(3.0, 4.0), 5.0);
	assert_close(Math::max(-1.0, 4.0), 4.0);
	assert_close(Math::min(-1.0, 4.0), -1.0);

	assert_close(Math::hypot_many(&[2.0, 3.0, 6.0]), 7.0);
	assert_close(Math::max_many(&[-1.0, 4.0, 2.0]), 4.0);
	assert_close(Math::min_many(&[-1.0, 4.0, 2.0]), -1.0);
}

#[test]
fn precise_sum() {
	if sum_precise_supported() {
		assert_close(Math::sum_precise(&precise_numbers()).unwrap(), 0.1);
	}
}

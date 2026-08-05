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
fn static_methods() {
	assert_close(Math::abs(-2.0), 2.0);
	assert_eq!(Math::clz32(1), 31);
	assert_close(Math::pow(2.0, 10.0), 1024.0);
}

#[test]
fn variadic() {
	assert_close(Math::hypot(3.0, 4.0), 5.0);
	assert_close(Math::hypot_many(&[2.0, 3.0, 6.0]), 7.0);
}

#[test]
fn precise_sum() {
	if sum_precise_supported() {
		assert_close(Math::sum_precise(&precise_numbers()).unwrap(), 0.1);
	}
}

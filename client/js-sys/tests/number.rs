use js_bindgen_test::test;
use js_sys::{BigInt, JsString, Number, Symbol};

fn assert_same_number(actual: f64, expected: f64) {
	assert_eq!(actual.to_bits(), expected.to_bits());
}

#[test]
fn number_methods() {
	let number = Number::new(JsString::from("12.5").as_ref()).unwrap();
	assert_same_number(number.value_of(), 12.5);
	assert!(!Number::is_integer(number.as_ref()));
	assert!(Number::is_finite(number.as_ref()));
	assert_eq!(
		String::from(&number.to_fixed_with_digits(1).unwrap()),
		"12.5"
	);
	assert_eq!(String::from(&number.to_string()), "12.5");
	assert_same_number(Number::parse_int("ff", 16), 255.0);
	assert_same_number(Number::parse_float("1.25px"), 1.25);
	assert!(number.to_fixed_with_digits(u8::MAX).is_err());
	assert!(Number::new(Symbol::new().as_ref()).is_err());
}

#[test]
fn bigint_methods() {
	let value = BigInt::new(JsString::from("18446744073709551615").as_ref()).unwrap();
	assert_eq!(String::from(&value.to_string()), "18446744073709551615");
	assert_eq!(
		String::from(&value.to_string_with_radix(16).unwrap()),
		"ffffffffffffffff"
	);
	assert_eq!(
		BigInt::as_uint_n(8.9, &value).unwrap().to_string(),
		JsString::from("255")
	);
	assert_eq!(
		BigInt::as_int_n(8.9, &value).unwrap().to_string(),
		JsString::from("-1")
	);
	for bits in [-1.0, f64::INFINITY] {
		assert!(BigInt::as_int_n(bits, &value).is_err());
		assert!(BigInt::as_uint_n(bits, &value).is_err());
	}
	assert!(value.to_string_with_radix(1).is_err());
	assert!(BigInt::new(JsString::from("not a bigint").as_ref()).is_err());
}

#[test]
fn number_constants() {
	assert_same_number(Number::MAX_SAFE_INTEGER, 9_007_199_254_740_991.0);
	assert_same_number(Number::MIN_SAFE_INTEGER, -9_007_199_254_740_991.0);
	assert_eq!(Number::MIN_VALUE.to_bits(), 1);
	assert!(Number::NAN.is_nan());
}

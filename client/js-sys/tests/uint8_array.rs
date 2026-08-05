#![expect(
	clippy::float_cmp,
	reason = "read and written counts are exact integer-valued JavaScript numbers"
)]

use js_bindgen_test::test;
use js_sys::hazard::JsCast;
use js_sys::{ArrayBuffer, Base64Alphabet, Base64DecodeOptions, Base64EncodeOptions, Uint8Array};

#[test]
fn base64() {
	let decode_options = Base64DecodeOptions::new();
	decode_options.set_alphabet(Base64Alphabet::Base64Url);
	let decoded = Uint8Array::from_base64_with_options("Love_you", &decode_options).unwrap();
	assert_eq!(decoded.to_vec().unwrap(), [46, 139, 222, 255, 42, 46]);

	let encode_options = Base64EncodeOptions::new();
	encode_options.set_alphabet(Base64Alphabet::Base64Url);
	encode_options.set_omit_padding(true);
	assert_eq!(
		decoded.to_base64_with_options(&encode_options).unwrap(),
		"Love_you"
	);
}

#[test]
fn set_from_base64_reports_progress() {
	let array = Uint8Array::new_with_length(8.0).unwrap();
	let result = array.set_from_base64("PGI+ TURO PC9i Pg==").unwrap();

	assert_eq!(result.read(), 9.0);
	assert_eq!(result.written(), 6.0);
	assert_eq!(array.to_vec().unwrap(), [60, 98, 62, 77, 68, 78, 0, 0]);
}

#[test]
fn hex() {
	let decoded = Uint8Array::from_hex("cafed00d").unwrap();
	assert_eq!(decoded.to_vec().unwrap(), [202, 254, 208, 13]);
	assert_eq!(decoded.to_hex().unwrap(), "cafed00d");
	assert!(Uint8Array::from_hex("bad").is_err());
}

#[test]
fn detached_arrays_return_errors() {
	let array = Uint8Array::from(&[1, 2, 3]);
	let buffer = ArrayBuffer::unchecked_from(array.buffer());
	buffer.transfer().unwrap();

	assert!(array.to_base64().is_err());
}

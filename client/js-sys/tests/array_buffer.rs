#![expect(
	clippy::float_cmp,
	reason = "buffer lengths and offsets are exact integer-valued JavaScript numbers"
)]

use js_bindgen_test::test;
use js_sys::{ArrayBuffer, ArrayBufferOptions, DataView, JsValue, SharedArrayBuffer, js_sys};

js_bindgen::embed_js!(
	module = "array_buffer",
	name = "invalidate_species",
	"(buffer) => {{ buffer.constructor = 0 }}",
);

#[js_sys]
extern "js-sys" {
	#[js_sys(js_embed = "invalidate_species")]
	fn invalidate_species(buffer: &SharedArrayBuffer);
}

#[test]
fn array_buffer_and_data_view() {
	assert!(DataView::new(&js_sys::JsValue::NULL).is_err());
	assert_eq!(ArrayBuffer::new(3.9).unwrap().byte_length(), 3.0);
	let buffer = ArrayBuffer::new(32.0).unwrap();
	assert_eq!(buffer.byte_length(), 32.0);
	assert!(!buffer.resizable());

	let view = DataView::new(buffer.as_ref()).unwrap();
	assert_eq!(view.byte_length().unwrap(), 32.0);
	assert_eq!(view.byte_offset().unwrap(), 0.0);
	assert_eq!(view.buffer(), JsValue::from(buffer.clone()));
	assert!(ArrayBuffer::is_view(view.as_ref()));

	view.set_uint8(0.0, u8::MAX).unwrap();
	view.set_int16_endian(1.0, i16::MIN, true).unwrap();
	view.set_uint32_endian(4.0, u32::MAX, true).unwrap();
	view.set_float64_endian(8.0, -0.0, true).unwrap();
	view.set_big_uint64_endian(16.0, u64::MAX, true).unwrap();
	view.set_uint16(24.0, 0x1234).unwrap();

	assert_eq!(view.get_uint8(0.0).unwrap(), u8::MAX);
	assert_eq!(view.get_int16_endian(1.0, true).unwrap(), i16::MIN);
	assert_eq!(view.get_uint32_endian(4.0, true).unwrap(), u32::MAX);
	assert!(
		view.get_float64_endian(8.0, true)
			.unwrap()
			.is_sign_negative()
	);
	assert_eq!(view.get_big_uint64_endian(16.0, true).unwrap(), u64::MAX);
	assert_eq!(view.get_uint8(24.0).unwrap(), 0x12);
	assert_eq!(view.get_uint8(25.0).unwrap(), 0x34);
	assert_eq!(view.get_uint16(24.0).unwrap(), 0x1234);
	assert_eq!(view.get_uint16(24.9).unwrap(), 0x1234);
	assert!(view.get_uint32(30.0).is_err());
	let window = DataView::new_with_offset_and_length(buffer.as_ref(), 4.0, 12.0).unwrap();
	assert_eq!(window.byte_offset().unwrap(), 4.0);
	assert_eq!(window.byte_length().unwrap(), 12.0);
	assert_eq!(window.get_uint32_endian(0.0, true).unwrap(), u32::MAX);
	assert!(DataView::new_with_offset(buffer.as_ref(), 33.0).is_err());

	let slice = buffer.slice_range(4.0, 12.0).unwrap();
	assert_eq!(slice.byte_length(), 8.0);

	let transferred = buffer.transfer().unwrap();
	assert!(buffer.detached());
	assert_eq!(transferred.byte_length(), 32.0);
	assert_eq!(
		DataView::new(transferred.as_ref())
			.unwrap()
			.get_uint32_endian(4.0, true)
			.unwrap(),
		u32::MAX
	);
	assert!(buffer.slice(0.0).is_err());
	assert!(view.byte_length().is_err());
	assert!(view.byte_offset().is_err());
}

#[test]
fn resizable_buffers() {
	let options = ArrayBufferOptions::new(64.0);
	assert_eq!(options.max_byte_length(), 64.0);

	let buffer = ArrayBuffer::new_with_options(16.0, &options).unwrap();
	assert!(buffer.resizable());
	buffer.resize(32.0).unwrap();
	assert_eq!(buffer.byte_length(), 32.0);
	let fixed = buffer.transfer_to_fixed_length_with_length(8.0).unwrap();
	assert!(buffer.detached());
	assert_eq!(fixed.byte_length(), 8.0);
	assert!(!fixed.resizable());

	// A shared worker may expose `SharedArrayBuffer` while rejecting growable
	// buffers, so construction itself is the capability check.
	if let Ok(shared) = SharedArrayBuffer::new_with_options(16.0, &options) {
		assert!(shared.growable());
		shared.grow(32.0).unwrap();
		assert_eq!(shared.byte_length(), 32.0);
		assert_eq!(shared.slice_range(4.0, 12.0).unwrap().byte_length(), 8.0);
		invalidate_species(&shared);
		assert!(shared.slice(0.0).is_err());
	}

	let too_small = ArrayBufferOptions::new(8.0);
	assert!(ArrayBuffer::new_with_options(16.0, &too_small).is_err());
	assert!(SharedArrayBuffer::new_with_options(16.0, &too_small).is_err());
}

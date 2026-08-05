#![expect(
	clippy::float_cmp,
	reason = "buffer lengths and offsets are exact integer-valued JavaScript numbers"
)]

use js_bindgen_test::test;
use js_sys::{ArrayBuffer, ArrayBufferOptions, DataView};

#[test]
fn array_buffer_and_data_view() {
	assert!(DataView::new(&js_sys::JsValue::NULL).is_err());
	let buffer = ArrayBuffer::new(8.0).unwrap();
	assert_eq!(buffer.byte_length(), 8.0);

	let view = DataView::new(buffer.as_ref()).unwrap();
	view.set_uint32_endian(0.0, u32::MAX, true).unwrap();
	assert_eq!(view.get_uint32_endian(0.0, true).unwrap(), u32::MAX);
	assert!(view.get_uint32(6.0).is_err());
}

#[test]
fn resizable_buffers() {
	let options = ArrayBufferOptions::new(64.0);
	assert_eq!(options.max_byte_length(), 64.0);

	let buffer = ArrayBuffer::new_with_options(16.0, &options).unwrap();
	assert!(buffer.resizable());
	buffer.resize(32.0).unwrap();
	assert_eq!(buffer.byte_length(), 32.0);
	assert!(buffer.transfer_to_fixed_length().is_ok());
}

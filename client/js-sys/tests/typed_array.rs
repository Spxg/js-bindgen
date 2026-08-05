#![expect(
	clippy::float_cmp,
	reason = "typed-array copies must preserve exact values"
)]

use js_bindgen_test::test;
use js_sys::hazard::JsCast;
use js_sys::{
	ArrayBuffer, BigInt64Array, BigUint64Array, Float16Array, Float32Array, Float64Array, Function,
	Int8Array, Int16Array, Int32Array, JsValue, Number, Symbol, TypedArray, TypedArrayCopyError,
	Uint8Array, Uint8ClampedArray, Uint16Array, Uint32Array, js_sys,
};

js_bindgen::embed_js!(
	module = "typed_array",
	name = "has_float16_array",
	"() => typeof Float16Array === 'function'",
);
js_bindgen::embed_js!(
	module = "typed_array",
	name = "mismatched_species",
	"() => new (class extends Uint32Array {{",
	"    static get [Symbol.species]() {{ return Uint8Array }}",
	"}})([1, 2, 3])",
);
js_bindgen::embed_js!(
	module = "typed_array",
	name = "fake",
	"() => ({{ slice: () => ({{}}), subarray: () => ({{}}) }})",
);
js_bindgen::embed_js!(module = "typed_array", name = "number", "(value) => value",);
js_bindgen::embed_js!(
	module = "typed_array",
	name = "shadow_length",
	"(array, length) => Object.defineProperty(array, 'length', {{ value: length }})",
);
js_bindgen::embed_js!(
	module = "typed_array",
	name = "has_resizable_array_buffer",
	"() => typeof ArrayBuffer.prototype.resize === 'function'",
);
js_bindgen::embed_js!(
	module = "typed_array",
	name = "resizable",
	"() => {{",
	"    const buffer = new ArrayBuffer(16, {{ maxByteLength: 16 }})",
	"    const array = new Uint32Array(buffer)",
	"    array.set([1, 2, 3, 4])",
	"    return array",
	"}}",
);
js_bindgen::embed_js!(
	module = "typed_array",
	name = "resize",
	"(array, byteLength) => array.buffer.resize(byteLength)",
);
js_bindgen::embed_js!(
	module = "typed_array",
	name = "hide_float16_array",
	"() => {{",
	"    const descriptor = Object.getOwnPropertyDescriptor(globalThis, 'Float16Array')",
	"    delete globalThis.Float16Array",
	"    return descriptor",
	"}}",
);
js_bindgen::embed_js!(
	module = "typed_array",
	name = "restore_float16_array",
	"(descriptor) => {{",
	"    if (descriptor === undefined) delete globalThis.Float16Array",
	"    else Object.defineProperty(globalThis, 'Float16Array', descriptor)",
	"}}",
);

#[js_sys]
extern "js-sys" {
	#[js_sys(js_embed = "has_float16_array")]
	fn has_float16_array() -> bool;

	#[js_sys(js_embed = "mismatched_species")]
	fn mismatched_species() -> Uint32Array;

	#[js_sys(js_embed = "fake")]
	fn fake_typed_array() -> Uint32Array;

	#[js_sys(js_embed = "number")]
	fn number(value: u32) -> JsValue;

	#[js_sys(js_embed = "shadow_length")]
	fn shadow_length(array: &Uint32Array, length: u32);

	#[js_sys(js_embed = "has_resizable_array_buffer")]
	fn has_resizable_array_buffer() -> bool;

	#[js_sys(js_embed = "resizable")]
	fn resizable() -> Uint32Array;

	#[js_sys(js_embed = "resize")]
	fn resize(array: &Uint32Array, byte_length: u32);

	#[js_sys(js_embed = "hide_float16_array")]
	fn hide_float16_array() -> JsValue;

	#[js_sys(js_embed = "restore_float16_array")]
	fn restore_float16_array(descriptor: &JsValue);
}

macro_rules! copy_tests {
	($name:ident, $array:ty, $values:expr, $replacement:expr) => {
		#[test]
		fn $name() {
			let values = $values;
			let array = <$array>::from(&values);
			assert_eq!(
				array.length(),
				f64::from(u32::try_from(values.len()).unwrap())
			);
			assert_eq!(array.to_vec().unwrap(), values);

			let replacement = $replacement;
			array.copy_from(&replacement).unwrap();
			let mut output = replacement;
			output.fill(replacement[0]);
			array.copy_to(&mut output).unwrap();
			assert_eq!(output, replacement);

			let mut wrong = [replacement[0]; 1];
			assert!(matches!(
				array.copy_to(&mut wrong),
				Err(TypedArrayCopyError::LengthMismatch)
			));
		}
	};
}

copy_tests!(int8, Int8Array, [-128_i8, 0, 127], [1_i8, 2, 3]);
copy_tests!(uint8, Uint8Array, [0_u8, 128, 255], [3_u8, 2, 1]);
copy_tests!(
	uint8_clamped,
	Uint8ClampedArray,
	[0_u8, 128, 255],
	[3_u8, 2, 1]
);
copy_tests!(int16, Int16Array, [i16::MIN, 0, i16::MAX], [1_i16, 2, 3]);
copy_tests!(
	uint16,
	Uint16Array,
	[0_u16, 0x8000, u16::MAX],
	[1_u16, 2, 3]
);
copy_tests!(int32, Int32Array, [i32::MIN, 0, i32::MAX], [1_i32, 2, 3]);
copy_tests!(
	uint32,
	Uint32Array,
	[0_u32, 0x8000_0000, u32::MAX],
	[1_u32, 2, 3]
);
copy_tests!(
	float32,
	Float32Array,
	[f32::NEG_INFINITY, -0.0, f32::INFINITY],
	[1.0_f32, 2.0, 3.0]
);
copy_tests!(
	float64,
	Float64Array,
	[f64::NEG_INFINITY, -0.0, f64::INFINITY],
	[1.0_f64, 2.0, 3.0]
);
copy_tests!(
	big_int64,
	BigInt64Array,
	[i64::MIN, 0, i64::MAX],
	[1_i64, 2, 3]
);
copy_tests!(
	big_uint64,
	BigUint64Array,
	[0_u64, 1 << 63, u64::MAX],
	[1_u64, 2, 3]
);

#[test]
fn float16() {
	if !has_float16_array() {
		assert!(Float16Array::species().is_err());
		return;
	}
	assert!(Float16Array::species().is_ok());

	let initial = [0x3c00_u16, 0xc000, 0x3555];
	let array = Float16Array::new_from_u16_slice(&initial).unwrap();
	assert_eq!(
		array.length(),
		f64::from(u32::try_from(initial.len()).unwrap())
	);
	assert_eq!(array.byte_length(), 6.0);
	assert_eq!(array.byte_offset(), 0.0);
	assert_eq!(Float16Array::BYTES_PER_ELEMENT, 2);
	assert_eq!(array.get_as_f32(0.0), Some(1.0));
	assert_eq!(array.get_as_f32(1.0), Some(-2.0));
	assert_eq!(array.at_as_f32(-2.0).unwrap(), Some(-2.0));
	assert_eq!(array.to_u16_vec().unwrap(), initial);

	array.set_index_from_f32(2.0, 0.5);
	assert_eq!(array.to_u16_vec().unwrap(), [0x3c00, 0xc000, 0x3800]);
	array.fill_with_f32_range(2.0, 1.0, 3.0).unwrap();
	assert_eq!(array.to_u16_vec().unwrap(), [0x3c00, 0x4000, 0x4000]);

	let replacement = [0x0001, 0x7bff, 0xfc00];
	array.copy_from_u16_slice(&replacement).unwrap();
	let mut copied = [0; 3];
	array.copy_to_u16_slice(&mut copied).unwrap();
	assert_eq!(copied, replacement);
	assert!(matches!(
		array.copy_to_u16_slice(&mut [0]),
		Err(TypedArrayCopyError::LengthMismatch)
	));

	let subarray = array.subarray_range(1.0, 3.0).unwrap();
	assert_eq!(subarray.to_u16_vec().unwrap(), [0x7bff, 0xfc00]);
	let copy = array.slice().unwrap();
	array.set_index_from_f32(0.0, 1.0);
	assert_eq!(copy.to_u16_vec().unwrap(), replacement);

	let positive = Function::new_with_args("value", "return value > 0").unwrap();
	assert_eq!(array.find_as_f32(&positive).unwrap(), Some(1.0));
	assert!(array.includes_f32(1.0).unwrap());
	assert_eq!(array.with_f32(1.0, 0.5).unwrap().get_as_f32(1.0), Some(0.5));
	assert_eq!(
		array.iter().collect::<Vec<_>>(),
		[1.0, 65504.0, -f32::INFINITY]
	);
}

#[test]
fn float16_errors_are_returned() {
	if !has_float16_array() {
		return;
	}

	assert!(Float16Array::new(Symbol::new().as_ref()).is_err());
	let array = Float16Array::new_from_u16_slice(&[0x3c00]).unwrap();
	let buffer = ArrayBuffer::unchecked_from(array.buffer());
	buffer.transfer().unwrap();

	assert!(array.at_as_f32(0.0).is_err());
	assert!(array.slice().is_err());
	assert!(matches!(
		array.copy_to_u16_slice(&mut [0]),
		Err(TypedArrayCopyError::JavaScript(_))
	));
}

#[test]
fn methods_and_buffer() {
	let buffer = ArrayBuffer::new(16.0).unwrap();
	assert!(Uint32Array::new_with_byte_offset(buffer.as_ref(), 1.0).is_err());
	assert!(Uint32Array::new(Symbol::new().as_ref()).is_err());
	let array = Uint32Array::new_with_byte_offset_and_length(buffer.as_ref(), 4.0, 2.0).unwrap();
	assert!(ArrayBuffer::is_view(array.as_ref()));
	assert_eq!(array.byte_offset(), 4.0);
	assert_eq!(array.byte_length(), 8.0);
	assert_eq!(array.length(), 2.0);

	array.set_index(0.0, u32::MAX);
	array.set_index(1.0, 7);
	assert_eq!(array.get(0.0), Some(u32::MAX));
	assert_eq!(array.at(-1.0).unwrap(), Some(7));
	assert_eq!(array.at(-1.9).unwrap(), Some(7));
	assert_eq!(array.get(2.0), None);

	let subarray = array.subarray_range(1.0, 2.0).unwrap();
	assert_eq!(subarray.to_vec().unwrap(), [7]);
	assert_eq!(array.subarray_from(1.0).unwrap().to_vec().unwrap(), [7]);
	let copy = array.slice().unwrap();
	array.set_index(0.0, 1);
	assert_eq!(copy.get(0.0), Some(u32::MAX));
	assert_eq!(array.slice_from(1.0).unwrap().to_vec().unwrap(), [7]);

	let source = Uint32Array::from(&[11, 13]);
	array.set(source.as_ref()).unwrap();
	assert_eq!(array.to_vec().unwrap(), [11, 13]);
	let source = Uint32Array::from(&[17]);
	array.set_with_offset(source.as_ref(), 1.0).unwrap();
	assert_eq!(array.to_vec().unwrap(), [11, 17]);
	array.fill_from(19, 1.0).unwrap();
	assert_eq!(array.to_vec().unwrap(), [11, 19]);

	let buffer: JsValue = array.buffer();
	assert_ne!(buffer, JsValue::NULL);
}

#[test]
fn static_methods_and_properties() {
	let values = [number(3), number(1), number(2)];
	let array = Uint32Array::of(&values).unwrap();
	assert_eq!(array.to_vec().unwrap(), [3, 1, 2]);

	let copy = Uint32Array::from_value(array.as_ref()).unwrap();
	assert_eq!(copy.to_vec().unwrap(), [3, 1, 2]);

	let double = Function::new_with_args("value", "return value * 2").unwrap();
	let doubled = Uint32Array::from_value_with_map(array.as_ref(), &double).unwrap();
	assert_eq!(doubled.to_vec().unwrap(), [6, 2, 4]);

	let scale = number(3);
	let multiply_by_this =
		Function::new_with_args("value", "return value * this.valueOf()").unwrap();
	let tripled =
		Uint32Array::from_value_with_map_and_this(array.as_ref(), &multiply_by_this, &scale)
			.unwrap();
	assert_eq!(tripled.to_vec().unwrap(), [9, 3, 6]);

	assert_eq!(array.bytes_per_element(), 4);
	assert_eq!(Uint32Array::BYTES_PER_ELEMENT, 4);
	assert_eq!(array.symbol_to_string_tag(), "Uint32Array");
	assert_eq!(
		AsRef::<JsValue>::as_ref(&array.constructor()),
		AsRef::<JsValue>::as_ref(&Uint32Array::species().unwrap())
	);
}

#[test]
fn missing_float16_species_is_reported() {
	let descriptor = hide_float16_array();
	let result = Float16Array::species();
	restore_float16_array(&descriptor);
	assert!(result.is_err());
}

#[test]
fn rust_and_javascript_iteration() {
	let array = Uint32Array::from(&[5, 8, 13]);
	let mut iter = array.iter();
	assert_eq!(iter.size_hint(), (0, Some(3)));
	assert_eq!(iter.next(), Some(5));
	assert_eq!(iter.next_back(), Some(13));
	assert_eq!(iter.next(), Some(8));
	assert_eq!(iter.next(), None);
	assert_eq!(iter.next_back(), None);

	assert_eq!(array.clone().into_iter().collect::<Vec<_>>(), [5, 8, 13]);
	assert_eq!((&array).into_iter().rev().collect::<Vec<_>>(), [13, 8, 5]);

	let values = array.values().unwrap();
	let copied = Uint32Array::from_value(values.as_ref()).unwrap();
	assert_eq!(copied.to_vec().unwrap(), [5, 8, 13]);
	let symbol_values = array.symbol_iterator().unwrap();
	let copied = Uint32Array::from_value(symbol_values.as_ref()).unwrap();
	assert_eq!(copied.to_vec().unwrap(), [5, 8, 13]);

	let keys: js_sys::JsIterator<Number<f64>> = array.keys().unwrap();
	let mut keys = keys.into_iter();
	assert_eq!(keys.next().unwrap().unwrap().value_of(), 0.0);
	assert_eq!(keys.next().unwrap().unwrap().value_of(), 1.0);
	assert_eq!(keys.next().unwrap().unwrap().value_of(), 2.0);

	let mut entries = array.entries().unwrap().into_iter();
	let first = entries.next().unwrap().unwrap();
	assert_eq!(
		Uint32Array::from_value(first.as_ref())
			.unwrap()
			.to_vec()
			.unwrap(),
		[0, 5]
	);
}

#[test]
fn copy_uses_the_intrinsic_typed_array_length() {
	let array = Uint32Array::from(&[3, 5]);
	shadow_length(&array, 1);

	let mut destination = [u32::MAX];
	assert!(matches!(
		array.copy_to(&mut destination),
		Err(TypedArrayCopyError::LengthMismatch)
	));
	assert_eq!(destination, [u32::MAX]);

	assert!(matches!(
		array.copy_from(&[8]),
		Err(TypedArrayCopyError::LengthMismatch)
	));
	assert_eq!(array.get(0.0), Some(3));
	assert_eq!(array.get(1.0), Some(5));
	assert_eq!(array.typed_array_length(), 2);
	assert_eq!(array.to_vec().unwrap(), [3, 5]);
	assert_eq!(array.iter().collect::<Vec<_>>(), [3, 5]);

	let array = Uint32Array::from(&[3, 5]);
	shadow_length(&array, 3);
	let mut destination = [u32::MAX; 3];
	assert!(matches!(
		array.copy_to(&mut destination),
		Err(TypedArrayCopyError::LengthMismatch)
	));
	assert_eq!(destination, [u32::MAX; 3]);
	assert_eq!(array.clone().into_iter().collect::<Vec<_>>(), [3, 5]);
}

#[test]
fn reverse_iteration_survives_resizable_buffer_shrinkage() {
	if !has_resizable_array_buffer() {
		return;
	}

	let array = resizable();
	let mut iter = array.iter();
	assert_eq!(iter.size_hint(), (0, Some(4)));
	resize(&array, 8);
	assert_eq!(iter.next_back(), Some(2));
	assert_eq!(iter.next_back(), Some(1));
	assert_eq!(iter.next_back(), None);
}

#[test]
fn species_cannot_change_the_element_type() {
	let array = mismatched_species();
	assert!(array.slice().is_err());
	assert!(array.slice_from(1.0).is_err());
	assert!(array.slice_range(1.0, 2.0).is_err());
	assert!(array.subarray().is_err());
	assert!(array.subarray_from(1.0).is_err());
	assert!(array.subarray_range(1.0, 2.0).is_err());
	let identity = Function::new_with_args("value", "return value").unwrap();
	assert!(array.filter(&identity).is_err());
	assert!(array.map(&identity).is_err());

	let fake = fake_typed_array();
	assert!(fake.slice().is_err());
	assert!(fake.subarray().is_err());
}

#[test]
fn detached_buffer_errors_are_returned() {
	let array = Uint8Array::from(&[1, 2, 3]);
	let buffer = ArrayBuffer::unchecked_from(array.buffer());
	buffer.transfer().unwrap();

	assert!(array.at(0.0).is_err());
	assert!(array.values().is_err());
	assert!(array.slice().is_err());
}

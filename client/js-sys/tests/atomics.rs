use js_bindgen_test::test;
use js_sys::{
	Atomics, BigInt64Array, BigUint64Array, Int8Array, Int16Array, Int32Array, JsString, JsValue,
	SharedArrayBuffer, Uint8Array, Uint16Array, Uint32Array,
};

#[test]
fn number_operations() {
	let Ok(buffer) = SharedArrayBuffer::new(16.0) else {
		return;
	};
	let array = Int32Array::new(buffer.as_ref()).unwrap();

	assert_eq!(Atomics::store(&array, 0.0, 10).unwrap(), 10);
	assert_eq!(Atomics::load(&array, 0.0).unwrap(), 10);
	assert_eq!(Atomics::load(&array, 0.9).unwrap(), 10);
	assert_eq!(Atomics::add(&array, 0.0, 5).unwrap(), 10);
	assert_eq!(Atomics::sub(&array, 0.0, 3).unwrap(), 15);
	assert_eq!(Atomics::exchange(&array, 0.0, 0b1010).unwrap(), 12);
	assert_eq!(Atomics::and(&array, 0.0, 0b1100).unwrap(), 0b1010);
	assert_eq!(Atomics::or(&array, 0.0, 0b0011).unwrap(), 0b1000);
	assert_eq!(Atomics::xor(&array, 0.0, 0b1111).unwrap(), 0b1011);
	assert_eq!(
		Atomics::compare_exchange(&array, 0.0, 0b0100, 7).unwrap(),
		0b0100
	);
	assert_eq!(Atomics::load(&array, 0.0).unwrap(), 7);
	assert!(Atomics::load(&array, array.length()).is_err());
}

#[test]
fn unsigned_values_preserve_their_range() {
	let Ok(buffer) = SharedArrayBuffer::new(4.0) else {
		return;
	};
	let array = Uint32Array::new(buffer.as_ref()).unwrap();

	assert_eq!(Atomics::store(&array, 0.0, u32::MAX).unwrap(), u32::MAX);
	assert_eq!(Atomics::add(&array, 0.0, 1).unwrap(), u32::MAX);
	assert_eq!(Atomics::load(&array, 0.0).unwrap(), 0);
}

#[test]
fn narrow_integer_arrays() {
	let Ok(buffer) = SharedArrayBuffer::new(8.0) else {
		return;
	};
	let int8 = Int8Array::new(buffer.as_ref()).unwrap();
	let uint8 = Uint8Array::new(buffer.as_ref()).unwrap();
	let int16 = Int16Array::new(buffer.as_ref()).unwrap();
	let uint16 = Uint16Array::new(buffer.as_ref()).unwrap();

	assert_eq!(Atomics::store(&int8, 0.0, i8::MIN).unwrap(), i8::MIN);
	assert_eq!(Atomics::load(&int8, 0.0).unwrap(), i8::MIN);
	assert_eq!(Atomics::store(&uint8, 1.0, u8::MAX).unwrap(), u8::MAX);
	assert_eq!(Atomics::load(&uint8, 1.0).unwrap(), u8::MAX);
	assert_eq!(Atomics::store(&int16, 1.0, i16::MIN).unwrap(), i16::MIN);
	assert_eq!(Atomics::load(&int16, 1.0).unwrap(), i16::MIN);
	assert_eq!(Atomics::store(&uint16, 2.0, u16::MAX).unwrap(), u16::MAX);
	assert_eq!(Atomics::load(&uint16, 2.0).unwrap(), u16::MAX);
}

#[test]
fn bigint_operations() {
	let Ok(signed_buffer) = SharedArrayBuffer::new(8.0) else {
		return;
	};
	let signed = BigInt64Array::new(signed_buffer.as_ref()).unwrap();
	assert_eq!(Atomics::store(&signed, 0.0, -2).unwrap(), -2);
	assert_eq!(Atomics::add(&signed, 0.0, 5).unwrap(), -2);
	assert_eq!(Atomics::load(&signed, 0.0).unwrap(), 3);

	let unsigned_buffer = SharedArrayBuffer::new(8.0).unwrap();
	let unsigned = BigUint64Array::new(unsigned_buffer.as_ref()).unwrap();
	assert_eq!(Atomics::store(&unsigned, 0.0, u64::MAX).unwrap(), u64::MAX);
	assert_eq!(Atomics::add(&unsigned, 0.0, 1).unwrap(), u64::MAX);
	assert_eq!(Atomics::load(&unsigned, 0.0).unwrap(), 0);
}

#[test]
#[expect(
	clippy::float_cmp,
	reason = "Atomics.notify returns an exact integral JavaScript Number"
)]
fn lock_free_and_notify() {
	assert!(Atomics::is_lock_free(4.0));

	let Ok(buffer) = SharedArrayBuffer::new(4.0) else {
		return;
	};
	let array = Int32Array::new(buffer.as_ref()).unwrap();
	assert_eq!(Atomics::notify(&array, 0.0).unwrap(), 0.0);
	assert_eq!(Atomics::notify_with_count(&array, 0.0, 1.0).unwrap(), 0.0);
}

#[test]
fn wait_immediate_results() {
	let Ok(buffer) = SharedArrayBuffer::new(8.0) else {
		return;
	};
	let array = Int32Array::new(buffer.as_ref()).unwrap();
	let bigint = BigInt64Array::new(buffer.as_ref()).unwrap();

	// Window agents cannot block. Use the non-matching call as the capability
	// check, and never issue an unbounded wait with a matching value.
	let Ok(not_equal) = Atomics::wait(&array, 0.0, 1) else {
		return;
	};
	assert_eq!(not_equal, "not-equal");
	assert_eq!(Atomics::wait_bigint(&bigint, 0.0, 1).unwrap(), "not-equal");
	assert_eq!(
		Atomics::wait_with_timeout(&array, 0.0, 0, 0.0).unwrap(),
		"timed-out"
	);
	assert_eq!(
		Atomics::wait_bigint_with_timeout(&bigint, 0.0, 0, 0.0).unwrap(),
		"timed-out"
	);
}

#[test]
fn wait_async_immediate_not_equal() {
	let Ok(buffer) = SharedArrayBuffer::new(8.0) else {
		return;
	};
	let array = Int32Array::new(buffer.as_ref()).unwrap();
	let bigint = BigInt64Array::new(buffer.as_ref()).unwrap();
	let expected = JsValue::from(JsString::from("not-equal"));

	for result in [
		Atomics::wait_async(&array, 0.0, 1).unwrap(),
		Atomics::wait_async_with_timeout(&array, 0.0, 1, 0.0).unwrap(),
		Atomics::wait_async_bigint(&bigint, 0.0, 1).unwrap(),
		Atomics::wait_async_bigint_with_timeout(&bigint, 0.0, 1, 0.0).unwrap(),
	] {
		assert!(!result.async_());
		assert_eq!(result.value(), expected);
	}
}

#![expect(
	clippy::float_cmp,
	reason = "buffer lengths are exact integral JavaScript Number values"
)]

use js_bindgen_test::test;
use js_sys::WebAssembly::{
	self, AddressType, CompileBuiltin, CompileError, CompileOptions, Exception, ExceptionOptions,
	Global, GlobalDescriptor, ImportExportKind, Instance, LinkError, Memory, MemoryDescriptor,
	Module, RuntimeError, SuspendError, Suspending, Table, TableDescriptor, TableElement, Tag,
	TagDescriptor, ValueType,
};
use js_sys::hazard::JsCast;
use js_sys::{
	ArrayBuffer, BigInt, Error, Function, JsString, JsValue, Number, Object, Promise, Reflect,
	Uint8Array, js_sys,
};

const EMPTY_MODULE: &[u8] = b"\0asm\x01\0\0\0";

const EXPORTED_FUNCTION_MODULE: &[u8] = &[
	0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, // Wasm header.
	0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f, // `() -> i32` type.
	0x03, 0x02, 0x01, 0x00, // One function using type 0.
	0x07, 0x0a, 0x01, 0x06, b'a', b'n', b's', b'w', b'e', b'r', 0x00, 0x00, // Export.
	0x0a, 0x06, 0x01, 0x04, 0x00, 0x41, 0x2a, 0x0b, // Return `i32.const 42`.
];

const IMPORTED_MEMORY_MODULE: &[u8] = &[
	0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, // Wasm header.
	0x02, 0x0f, 0x01, 0x03, b'e', b'n', b'v', 0x06, b'm', b'e', b'm', b'o', b'r', b'y', 0x02, 0x00,
	0x01, // Import `env.memory` with a minimum of one page.
];

js_bindgen::embed_js!(
	module = "webassembly",
	name = "has_table64",
	"() => {{",
	"    try {{",
	"        new WebAssembly.Table({{ element: 'externref', initial: 0n, address: 'i64' }})",
	"        return true",
	"    }} catch {{",
	"        return false",
	"    }}",
	"}}",
);

#[js_sys]
extern "js-sys" {
	#[js_sys(js_embed = "has_table64")]
	fn has_table64() -> bool;
}

fn wasm_bytes(bytes: &[u8]) -> Uint8Array {
	Uint8Array::from(bytes)
}

fn number(value: &str) -> JsValue {
	Number::new(JsString::from(value).as_ref()).unwrap().into()
}

fn bigint(value: &str) -> JsValue {
	BigInt::new(JsString::from(value).as_ref()).unwrap().into()
}

fn assert_number(value: &JsValue, expected: &str) {
	assert_eq!(JsString::new(value).unwrap(), expected);
}

fn assert_answer(instance: &Instance) {
	let exports = instance.exports();
	let answer = Function::unchecked_from(Reflect::get_str(exports.as_ref(), "answer").unwrap());
	let result = answer.call(&JsValue::UNDEFINED, &[]).unwrap();
	assert_number(&result, "42");
}

fn has_webassembly_member(name: &str) -> bool {
	let namespace = Reflect::get_str(&js_sys::global_this(), "WebAssembly").unwrap();
	Reflect::has_str(&namespace, name).unwrap()
}

#[test]
fn namespace_validation_and_compile_options() {
	let bytes = wasm_bytes(EMPTY_MODULE);
	assert!(WebAssembly::validate(bytes.as_ref()).unwrap());
	assert!(!WebAssembly::validate(wasm_bytes(&[0]).as_ref()).unwrap());
	assert!(WebAssembly::validate(&JsValue::NULL).is_err());

	let options = CompileOptions::new();
	assert!(options.builtins().is_none());
	assert!(options.imported_string_constants().is_none());
	options.set_builtins(&[CompileBuiltin::JsString]);
	options.set_imported_string_constants("string_constants");
	assert_eq!(options.builtins(), Some(vec![CompileBuiltin::JsString]));
	assert_eq!(
		options.imported_string_constants(),
		Some(JsString::from("string_constants"))
	);
	assert!(WebAssembly::validate_with_options(bytes.as_ref(), &options).unwrap());
}

#[test]
fn module_and_instance() {
	let bytes = wasm_bytes(EXPORTED_FUNCTION_MODULE);
	let module = Module::new(bytes.as_ref()).unwrap();
	let options = CompileOptions::new();
	Module::new_with_options(bytes.as_ref(), &options).unwrap();
	assert!(Module::new(wasm_bytes(&[0]).as_ref()).is_err());

	let exports = Module::exports(&module).unwrap();
	assert_eq!(exports.length(), 1);
	let descriptor = exports.get_unchecked(0);
	assert_eq!(descriptor.name(), "answer");
	assert_eq!(descriptor.kind(), Some(ImportExportKind::Function));
	assert_eq!(
		Module::custom_sections(&module, "missing")
			.unwrap()
			.length(),
		0
	);

	let instance = Instance::new(&module).unwrap();
	assert_answer(&instance);
	assert!(Instance::new_with_imports(&module, &Object::new()).is_ok());

	let imported = Module::new(wasm_bytes(IMPORTED_MEMORY_MODULE).as_ref()).unwrap();
	let imports = Module::imports(&imported).unwrap();
	assert_eq!(imports.length(), 1);
	let descriptor = imports.get_unchecked(0);
	assert_eq!(descriptor.module(), "env");
	assert_eq!(descriptor.name(), "memory");
	assert_eq!(descriptor.kind(), Some(ImportExportKind::Memory));
	assert!(Instance::new(&imported).is_err());
}

#[test]
async fn namespace_compilation_and_instantiation() {
	let bytes = wasm_bytes(EXPORTED_FUNCTION_MODULE);
	let module = WebAssembly::compile(bytes.as_ref()).await.unwrap();
	assert_answer(&WebAssembly::instantiate_module(&module).await.unwrap());

	let source = WebAssembly::instantiate_bytes(bytes.as_ref())
		.await
		.unwrap();
	assert_eq!(Module::exports(&source.module()).unwrap().length(), 1);
	assert_answer(&source.instance());

	let options = CompileOptions::new();
	options.set_builtins(&[CompileBuiltin::JsString]);
	let module = WebAssembly::compile_with_options(bytes.as_ref(), &options)
		.await
		.unwrap();
	assert_answer(
		&WebAssembly::instantiate_module_with_imports(&module, &Object::new())
			.await
			.unwrap(),
	);
	let source = WebAssembly::instantiate_bytes_with_imports_and_options(
		bytes.as_ref(),
		&Object::new(),
		&options,
	)
	.await
	.unwrap();
	assert_answer(&source.instance());

	assert!(
		WebAssembly::compile(wasm_bytes(&[0]).as_ref())
			.await
			.is_err()
	);
	let imported = WebAssembly::compile(wasm_bytes(IMPORTED_MEMORY_MODULE).as_ref())
		.await
		.unwrap();
	assert!(WebAssembly::instantiate_module(&imported).await.is_err());
}

#[test]
fn memory() {
	let descriptor = MemoryDescriptor::new(1);
	assert_number(&descriptor.initial(), "1");
	assert!(descriptor.maximum().is_none());
	assert!(descriptor.shared().is_none());
	descriptor.set_maximum(2);
	assert_number(&descriptor.maximum().unwrap(), "2");

	let memory = Memory::new(&descriptor).unwrap();
	let buffer = ArrayBuffer::unchecked_from(memory.buffer());
	assert_eq!(buffer.byte_length(), 65_536.0);
	assert_eq!(memory.grow(1).unwrap(), 1);
	let buffer = ArrayBuffer::unchecked_from(memory.buffer());
	assert_eq!(buffer.byte_length(), 131_072.0);
	assert!(memory.grow(1).is_err());
}

#[test]
fn memory_buffer_modes() {
	let descriptor = MemoryDescriptor::new(1);
	descriptor.set_maximum(2);
	let memory = Memory::new(&descriptor).unwrap();

	let resizable = ArrayBuffer::unchecked_from(memory.to_resizable_buffer().unwrap());
	assert!(resizable.resizable());
	assert_eq!(resizable.byte_length(), 65_536.0);
	assert_eq!(memory.grow(1).unwrap(), 1);
	assert_eq!(resizable.byte_length(), 131_072.0);

	let fixed = ArrayBuffer::unchecked_from(memory.to_fixed_length_buffer());
	assert!(!fixed.resizable());
	assert_eq!(fixed.byte_length(), 131_072.0);
}

#[test]
fn memory64() {
	let descriptor = MemoryDescriptor::new64(1);
	assert_eq!(descriptor.address(), Some(AddressType::I64));
	assert_eq!(JsString::new(&descriptor.initial()).unwrap(), "1");
	descriptor.set_maximum64(2);
	assert_eq!(JsString::new(&descriptor.maximum().unwrap()).unwrap(), "2");

	let memory = Memory::new(&descriptor).unwrap();
	assert_eq!(memory.grow64(1).unwrap(), 1);
	assert_eq!(
		ArrayBuffer::unchecked_from(memory.buffer()).byte_length(),
		131_072.0
	);
	assert!(memory.grow64(1).is_err());
}

#[test]
fn table() {
	let descriptor = TableDescriptor::new(TableElement::ExternRef, 1);
	assert_eq!(descriptor.element(), Some(TableElement::ExternRef));
	assert_number(&descriptor.initial(), "1");
	descriptor.set_maximum(2);

	let table = Table::new(&descriptor).unwrap();
	assert_eq!(table.length(), 1);
	assert_eq!(table.get(0).unwrap(), JsValue::UNDEFINED);

	let value = JsValue::from(JsString::from("stored"));
	table.set_with_value(0, &value).unwrap();
	assert_eq!(table.get(0).unwrap(), value);
	table.set(0).unwrap();
	assert_eq!(table.get(0).unwrap(), JsValue::UNDEFINED);

	let grown = JsValue::from(JsString::from("grown"));
	assert_eq!(table.grow_with_value(1, &grown).unwrap(), 1);
	assert_eq!(table.length(), 2);
	assert_eq!(table.get(1).unwrap(), grown);
	assert!(table.grow(1).is_err());
}

#[test]
fn table_omitted_value() {
	let table = Table::new(&TableDescriptor::new(TableElement::AnyFunc, 1)).unwrap();
	let module = Module::new(wasm_bytes(EXPORTED_FUNCTION_MODULE).as_ref()).unwrap();
	let instance = Instance::new(&module).unwrap();
	let answer = Reflect::get_str(instance.exports().as_ref(), "answer").unwrap();
	table.set_with_value(0, &answer).unwrap();
	assert_eq!(table.get(0).unwrap(), answer);

	table.set(0).unwrap();
	assert_eq!(table.get(0).unwrap(), JsValue::NULL);
}

#[test]
fn table64_if_supported() {
	if !has_table64() {
		return;
	}

	let descriptor = TableDescriptor::new64(TableElement::ExternRef, 1);
	descriptor.set_maximum64(2);
	let table = Table::new(&descriptor).unwrap();

	assert_eq!(table.length64(), 1);
	let value = JsValue::from(JsString::from("stored"));
	table.set64_with_value(0, &value).unwrap();
	assert_eq!(table.get64(0).unwrap(), value);
	table.set64(0).unwrap();
	assert_eq!(table.get64(0).unwrap(), JsValue::UNDEFINED);
	assert_eq!(table.grow64(1).unwrap(), 1);
	assert!(table.grow64(1).is_err());
}

#[test]
fn global() {
	let descriptor = GlobalDescriptor::new(ValueType::I32);
	assert_eq!(descriptor.value_type(), Some(ValueType::I32));
	assert!(descriptor.mutable().is_none());
	descriptor.set_mutable(true);
	assert_eq!(descriptor.mutable(), Some(true));

	let global = Global::new_with_value(&descriptor, &number("7")).unwrap();
	assert_number(&global.value().unwrap(), "7");
	global.set_value(&number("9")).unwrap();
	assert_number(&global.value_of().unwrap(), "9");

	let immutable = Global::new(&GlobalDescriptor::new(ValueType::I32)).unwrap();
	assert_number(&immutable.value().unwrap(), "0");
	assert!(immutable.set_value(&number("1")).is_err());
}

#[test]
fn global_i64() {
	let descriptor = GlobalDescriptor::new(ValueType::I64);
	descriptor.set_mutable(true);
	let global = Global::new_with_value(&descriptor, &bigint("7")).unwrap();
	assert_eq!(JsString::new(&global.value().unwrap()).unwrap(), "7");
	global.set_value(&bigint("9")).unwrap();
	assert_eq!(JsString::new(&global.value_of().unwrap()).unwrap(), "9");
	assert!(Global::new_with_value(&descriptor, &number("1")).is_err());
}

fn assert_error<T: AsRef<Error> + AsRef<Object>>(error: &T, name: &str) {
	let error: &Error = error.as_ref();
	assert_eq!(error.name(), name);
	assert_eq!(error.message(), "failed");
}

#[test]
fn webassembly_errors() {
	assert_error(&CompileError::new("failed"), "CompileError");
	assert_error(&LinkError::new("failed"), "LinkError");
	assert_error(&RuntimeError::new("failed"), "RuntimeError");
	if has_webassembly_member("SuspendError") {
		assert_error(&SuspendError::new("failed"), "SuspendError");
	}
}

#[test]
fn tag_and_exception() {
	let descriptor = TagDescriptor::new(&[ValueType::I32]);
	let parameters = descriptor.parameters();
	assert_eq!(parameters.length(), 1);
	assert_eq!(parameters.get_unchecked(0), "i32");

	let tag = Tag::new(&descriptor).unwrap();
	let other_tag = Tag::new(&descriptor).unwrap();
	let payload = [number("7")];
	let exception = Exception::new(&tag, &payload).unwrap();
	assert!(exception.is(&tag));
	assert!(!exception.is(&other_tag));
	assert_number(&exception.get_arg(&tag, 0).unwrap(), "7");
	assert!(exception.get_arg(&other_tag, 0).is_err());

	let options = ExceptionOptions::new();
	options.set_trace_stack(true);
	assert_eq!(options.trace_stack(), Some(true));
	let traced = Exception::new_with_options(&tag, &payload, &options).unwrap();
	assert_number(&traced.get_arg(&tag, 0).unwrap(), "7");

	let js_tag = WebAssembly::js_tag();
	assert_eq!(js_tag, WebAssembly::js_tag());
}

#[test]
async fn jspi_surface() {
	if !has_webassembly_member("Suspending") || !has_webassembly_member("promising") {
		return;
	}

	let async_function = Function::new_no_args("return Promise.resolve(1)").unwrap();
	Suspending::new(&async_function).unwrap();
	assert!(WebAssembly::promising(&async_function).is_err());

	let module = Module::new(wasm_bytes(EXPORTED_FUNCTION_MODULE).as_ref()).unwrap();
	let instance = Instance::new(&module).unwrap();
	let answer =
		Function::unchecked_from(Reflect::get_str(instance.exports().as_ref(), "answer").unwrap());
	let promising = WebAssembly::promising(&answer).unwrap();
	let promise = Promise::unchecked_from(promising.call(&JsValue::UNDEFINED, &[]).unwrap());
	assert_number(&promise.await.unwrap(), "42");
}

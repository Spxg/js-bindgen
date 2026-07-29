fn generated_js(input: syn::ItemForeignMod) -> String {
	let output = crate::r#macro::internal(
		proc_macro2::TokenStream::new(),
		input,
		Some("test_crate"),
		None,
	)
	.unwrap()
	.into_items()
	.unwrap();
	let output = prettyplease::unparse(&syn::File {
		shebang: None,
		attrs: Vec::new(),
		items: output,
	});

	let dir = tempfile::tempdir().unwrap();
	let (_, js, _) = super::inner(dir.path(), &output).unwrap();
	js.unwrap()
}

#[test]
fn method() {
	test!(
		{},
		{
			extern "js-sys" {
				pub fn test(self: &JsTest);
			}
		},
		{
			impl JsTest {
				pub fn test(self: &JsTest) {
					unsafe extern "C" {
						#[link_name = "test_crate.JsTest.test"]
						fn test(
							arg0_0: ::js_sys::r#macro::InputSlot1<&::js_sys::JsValue>,
							arg0_1: ::js_sys::r#macro::InputSlot2<&::js_sys::JsValue>,
							arg0_2: ::js_sys::r#macro::InputSlot3<&::js_sys::JsValue>,
							arg0_3: ::js_sys::r#macro::InputSlot4<&::js_sys::JsValue>,
						);
					}

					{
						let (arg0_0, arg0_1, arg0_2, arg0_3) = unsafe {
							::js_sys::r#macro::split_input_as::<&::js_sys::JsValue>(self)
						};
						unsafe { test(arg0_0, arg0_1, arg0_2, arg0_3) }
					};
				}
			}
			const _: () = {
				fn import_sections() {
					const IMPORTS: &[::js_sys::r#macro::ImportDescriptor] =
						&[::js_sys::r#macro::ImportDescriptor::new(
							"test_crate",
							"JsTest.test",
							"test_crate.JsTest.test",
							&[::js_sys::r#macro::import_input::<&::js_sys::JsValue>(
								"arg0",
							)],
							::core::option::Option::None,
							::core::option::Option::Some(::js_sys::r#macro::ImportJs {
								direct_wrapper: true,
								direct_call: "arg0_0.test()",
								indirect_call: "arg0_0.test()",
								required_embeds: &[::js_sys::r#macro::js_input_embed::<
									&::js_sys::JsValue,
								>()],
							}),
						)];
					const WAT_CAPACITY: ::core::primitive::usize =
						::js_sys::r#macro::import_wat_batch_capacity(IMPORTS);
					#[used]
					#[unsafe(link_section = "js_bindgen.wat")]
					static WAT_SECTION: ::js_sys::r#macro::ImportBatchSection<WAT_CAPACITY> =
						::js_sys::r#macro::import_wat_batch::<WAT_CAPACITY>(IMPORTS);
					const JS_CAPACITY: ::core::primitive::usize =
						::js_sys::r#macro::import_js_batch_capacity(IMPORTS);
					#[used]
					#[unsafe(link_section = "js_bindgen.import")]
					static JS_SECTION: ::js_sys::r#macro::ImportBatchSection<JS_CAPACITY> =
						::js_sys::r#macro::import_js_batch::<JS_CAPACITY>(IMPORTS);
				}
			};
		},
		"(import \"test_crate\" \"JsTest.test\" (func $test_crate.import.JsTest.test (@sym (name \
		 \"test_crate.import.JsTest.test\")) (param externref)))
		(import \"js_sys\" \"externref.table\" (table $js_sys.import.externref.table (@sym (name \
		 \"js_sys.externref.table\")) 2 externref))
		(func $test_crate.JsTest.test (@sym) (param $arg0_0 i32)
		  local.get $arg0_0
		  table.get $js_sys.import.externref.table (@reloc)
		  call $test_crate.import.JsTest.test (@reloc)
		)",
		"(arg0_0) => arg0_0.test()",
	);
}

#[test]
fn method_par() {
	test!(
		{},
		{
			extern "js-sys" {
				pub fn test(self: &JsTest, par1: &JsValue, par2: &JsValue);
			}
		},
		{
			impl JsTest {
				pub fn test(self: &JsTest, par1: &JsValue, par2: &JsValue) {
					unsafe extern "C" {
						#[link_name = "test_crate.JsTest.test"]
						fn test(
							arg0_0: ::js_sys::r#macro::InputSlot1<&::js_sys::JsValue>,
							arg0_1: ::js_sys::r#macro::InputSlot2<&::js_sys::JsValue>,
							arg0_2: ::js_sys::r#macro::InputSlot3<&::js_sys::JsValue>,
							arg0_3: ::js_sys::r#macro::InputSlot4<&::js_sys::JsValue>,
							arg1_0: ::js_sys::r#macro::InputSlot1<&JsValue>,
							arg1_1: ::js_sys::r#macro::InputSlot2<&JsValue>,
							arg1_2: ::js_sys::r#macro::InputSlot3<&JsValue>,
							arg1_3: ::js_sys::r#macro::InputSlot4<&JsValue>,
							arg2_0: ::js_sys::r#macro::InputSlot1<&JsValue>,
							arg2_1: ::js_sys::r#macro::InputSlot2<&JsValue>,
							arg2_2: ::js_sys::r#macro::InputSlot3<&JsValue>,
							arg2_3: ::js_sys::r#macro::InputSlot4<&JsValue>,
						);
					}

					{
						let (arg0_0, arg0_1, arg0_2, arg0_3) = unsafe {
							::js_sys::r#macro::split_input_as::<&::js_sys::JsValue>(self)
						};
						let (arg1_0, arg1_1, arg1_2, arg1_3) =
							::js_sys::r#macro::split_input::<&JsValue>(par1);
						let (arg2_0, arg2_1, arg2_2, arg2_3) =
							::js_sys::r#macro::split_input::<&JsValue>(par2);
						unsafe {
							test(
								arg0_0, arg0_1, arg0_2, arg0_3, arg1_0, arg1_1, arg1_2, arg1_3,
								arg2_0, arg2_1, arg2_2, arg2_3,
							)
						}
					};
				}
			}
			const _: () = {
				fn import_sections() {
					const IMPORTS: &[::js_sys::r#macro::ImportDescriptor] =
						&[::js_sys::r#macro::ImportDescriptor::new(
							"test_crate",
							"JsTest.test",
							"test_crate.JsTest.test",
							&[
								::js_sys::r#macro::import_input::<&::js_sys::JsValue>("arg0"),
								::js_sys::r#macro::import_input::<&JsValue>("arg1"),
								::js_sys::r#macro::import_input::<&JsValue>("arg2"),
							],
							::core::option::Option::None,
							::core::option::Option::Some(::js_sys::r#macro::ImportJs {
								direct_wrapper: true,
								direct_call: "arg0_0.test(arg1_0, arg2_0)",
								indirect_call: "arg0_0.test(arg1_0, arg2_0)",
								required_embeds: &[
									::js_sys::r#macro::js_input_embed::<&::js_sys::JsValue>(),
									::js_sys::r#macro::js_input_embed::<&JsValue>(),
								],
							}),
						)];
					const WAT_CAPACITY: ::core::primitive::usize =
						::js_sys::r#macro::import_wat_batch_capacity(IMPORTS);
					#[used]
					#[unsafe(link_section = "js_bindgen.wat")]
					static WAT_SECTION: ::js_sys::r#macro::ImportBatchSection<WAT_CAPACITY> =
						::js_sys::r#macro::import_wat_batch::<WAT_CAPACITY>(IMPORTS);
					const JS_CAPACITY: ::core::primitive::usize =
						::js_sys::r#macro::import_js_batch_capacity(IMPORTS);
					#[used]
					#[unsafe(link_section = "js_bindgen.import")]
					static JS_SECTION: ::js_sys::r#macro::ImportBatchSection<JS_CAPACITY> =
						::js_sys::r#macro::import_js_batch::<JS_CAPACITY>(IMPORTS);
				}
			};
		},
		"(import \"test_crate\" \"JsTest.test\" (func $test_crate.import.JsTest.test (@sym (name \
		 \"test_crate.import.JsTest.test\")) (param externref externref externref)))
		(import \"js_sys\" \"externref.table\" (table $js_sys.import.externref.table (@sym (name \
		 \"js_sys.externref.table\")) 2 externref))
		(func $test_crate.JsTest.test (@sym) (param $arg0_0 i32) (param $arg1_0 i32) (param $arg2_0 i32)
		  local.get $arg0_0
		  table.get $js_sys.import.externref.table (@reloc)
		  local.get $arg1_0
		  table.get $js_sys.import.externref.table (@reloc)
		  local.get $arg2_0
		  table.get $js_sys.import.externref.table (@reloc)
		  call $test_crate.import.JsTest.test (@reloc)
		)",
		"(arg0_0, arg1_0, arg2_0) => arg0_0.test(arg1_0, arg2_0)",
	);
}

#[test]
fn variadic() {
	let input = syn::parse_quote! {
		extern "js-sys" {
			#[js_sys(variadic)]
			pub fn push(self: &JsTest, first: &JsValue, rest: &[JsValue]);
		}
	};

	assert_eq!(
		generated_js(input),
		"(arg0_0, arg1_0, arg2_0, arg2_1) => {\n    \
		 arg2_0 = this.#jsEmbed.js_sys['array.js_value.decode'](arg2_0, arg2_1)\n\
		 arg0_0.push(arg1_0, ...arg2_0)\n}"
	);
}

#[test]
fn global_variadic() {
	let input = syn::parse_quote! {
		extern "js-sys" {
			#[js_sys(variadic)]
			pub fn call(values: &JsArray);
		}
	};

	assert_eq!(
		generated_js(input),
		"(arg0_0) => globalThis.call(...arg0_0)"
	);
}

#[test]
fn getter() {
	test!(
		{},
		{
			extern "js-sys" {
				#[js_sys(getter)]
				pub fn test(self: &JsTest) -> JsValue;
			}
		},
		{
			impl JsTest {
				pub fn test(self: &JsTest) -> JsValue {
					unsafe extern "C" {
						#[link_name = "test_crate.JsTest.test"]
						fn test(
							arg0_0: ::js_sys::r#macro::InputSlot1<&::js_sys::JsValue>,
							arg0_1: ::js_sys::r#macro::InputSlot2<&::js_sys::JsValue>,
							arg0_2: ::js_sys::r#macro::InputSlot3<&::js_sys::JsValue>,
							arg0_3: ::js_sys::r#macro::InputSlot4<&::js_sys::JsValue>,
						) -> ::js_sys::r#macro::OutputRet<JsValue>;
					}

					::js_sys::r#macro::join_output({
						let (arg0_0, arg0_1, arg0_2, arg0_3) = unsafe {
							::js_sys::r#macro::split_input_as::<&::js_sys::JsValue>(self)
						};
						unsafe { test(arg0_0, arg0_1, arg0_2, arg0_3) }
					})
				}
			}
			const _: () = {
				fn import_sections() {
					const IMPORTS: &[::js_sys::r#macro::ImportDescriptor] =
						&[::js_sys::r#macro::ImportDescriptor::new(
							"test_crate",
							"JsTest.test",
							"test_crate.JsTest.test",
							&[::js_sys::r#macro::import_input::<&::js_sys::JsValue>(
								"arg0",
							)],
							::core::option::Option::Some(
								::js_sys::r#macro::import_output::<JsValue>(),
							),
							::core::option::Option::Some(::js_sys::r#macro::ImportJs {
								direct_wrapper: true,
								direct_call: "arg0_0.test",
								indirect_call: "arg0_0.test",
								required_embeds: &[
									::js_sys::r#macro::js_input_embed::<&::js_sys::JsValue>(),
									::js_sys::r#macro::js_output_embed::<JsValue>(),
									::js_sys::r#macro::js_result_embed::<JsValue>(),
								],
							}),
						)];
					const WAT_CAPACITY: ::core::primitive::usize =
						::js_sys::r#macro::import_wat_batch_capacity(IMPORTS);
					#[used]
					#[unsafe(link_section = "js_bindgen.wat")]
					static WAT_SECTION: ::js_sys::r#macro::ImportBatchSection<WAT_CAPACITY> =
						::js_sys::r#macro::import_wat_batch::<WAT_CAPACITY>(IMPORTS);
					const JS_CAPACITY: ::core::primitive::usize =
						::js_sys::r#macro::import_js_batch_capacity(IMPORTS);
					#[used]
					#[unsafe(link_section = "js_bindgen.import")]
					static JS_SECTION: ::js_sys::r#macro::ImportBatchSection<JS_CAPACITY> =
						::js_sys::r#macro::import_js_batch::<JS_CAPACITY>(IMPORTS);
				}
			};
		},
		"(import \"test_crate\" \"JsTest.test\" (func $test_crate.import.JsTest.test (@sym (name \
		 \"test_crate.import.JsTest.test\")) (param externref) (result externref)))
		(import \"js_sys\" \"externref.table\" (table $js_sys.import.externref.table (@sym (name \
		 \"js_sys.externref.table\")) 2 externref))
		(import \"env\" \"js_sys.externref.next\" (func $js_sys.externref.next (@sym) (result i32)))
		(func $test_crate.JsTest.test (@sym) (param $arg0_0 i32) (result i32)
		  (local $js_sys.externref.value externref)
		  (local $js_sys.externref.index i32)
		  local.get $arg0_0
		  table.get $js_sys.import.externref.table (@reloc)
		  call $test_crate.import.JsTest.test (@reloc)
		  local.set $js_sys.externref.value
		  call $js_sys.externref.next (@reloc)
		  local.tee $js_sys.externref.index
		  local.get $js_sys.externref.value
		  table.set $js_sys.import.externref.table (@reloc)
		  local.get $js_sys.externref.index
		)",
		"(arg0_0) => arg0_0.test",
	);
}

#[test]
fn setter() {
	test!(
		{},
		{
			extern "js-sys" {
				#[js_sys(setter)]
				pub fn test(self: &JsTest, value: &JsValue);
			}
		},
		{
			impl JsTest {
				pub fn test(self: &JsTest, value: &JsValue) {
					unsafe extern "C" {
						#[link_name = "test_crate.JsTest.test"]
						fn test(
							arg0_0: ::js_sys::r#macro::InputSlot1<&::js_sys::JsValue>,
							arg0_1: ::js_sys::r#macro::InputSlot2<&::js_sys::JsValue>,
							arg0_2: ::js_sys::r#macro::InputSlot3<&::js_sys::JsValue>,
							arg0_3: ::js_sys::r#macro::InputSlot4<&::js_sys::JsValue>,
							arg1_0: ::js_sys::r#macro::InputSlot1<&JsValue>,
							arg1_1: ::js_sys::r#macro::InputSlot2<&JsValue>,
							arg1_2: ::js_sys::r#macro::InputSlot3<&JsValue>,
							arg1_3: ::js_sys::r#macro::InputSlot4<&JsValue>,
						);
					}

					{
						let (arg0_0, arg0_1, arg0_2, arg0_3) = unsafe {
							::js_sys::r#macro::split_input_as::<&::js_sys::JsValue>(self)
						};
						let (arg1_0, arg1_1, arg1_2, arg1_3) =
							::js_sys::r#macro::split_input::<&JsValue>(value);
						unsafe {
							test(
								arg0_0, arg0_1, arg0_2, arg0_3, arg1_0, arg1_1, arg1_2, arg1_3,
							)
						}
					};
				}
			}
			const _: () = {
				fn import_sections() {
					const IMPORTS: &[::js_sys::r#macro::ImportDescriptor] =
						&[::js_sys::r#macro::ImportDescriptor::new(
							"test_crate",
							"JsTest.test",
							"test_crate.JsTest.test",
							&[
								::js_sys::r#macro::import_input::<&::js_sys::JsValue>("arg0"),
								::js_sys::r#macro::import_input::<&JsValue>("arg1"),
							],
							::core::option::Option::None,
							::core::option::Option::Some(::js_sys::r#macro::ImportJs {
								direct_wrapper: true,
								direct_call: "arg0_0.test = arg1_0",
								indirect_call: "arg0_0.test = arg1_0",
								required_embeds: &[
									::js_sys::r#macro::js_input_embed::<&::js_sys::JsValue>(),
									::js_sys::r#macro::js_input_embed::<&JsValue>(),
								],
							}),
						)];
					const WAT_CAPACITY: ::core::primitive::usize =
						::js_sys::r#macro::import_wat_batch_capacity(IMPORTS);
					#[used]
					#[unsafe(link_section = "js_bindgen.wat")]
					static WAT_SECTION: ::js_sys::r#macro::ImportBatchSection<WAT_CAPACITY> =
						::js_sys::r#macro::import_wat_batch::<WAT_CAPACITY>(IMPORTS);
					const JS_CAPACITY: ::core::primitive::usize =
						::js_sys::r#macro::import_js_batch_capacity(IMPORTS);
					#[used]
					#[unsafe(link_section = "js_bindgen.import")]
					static JS_SECTION: ::js_sys::r#macro::ImportBatchSection<JS_CAPACITY> =
						::js_sys::r#macro::import_js_batch::<JS_CAPACITY>(IMPORTS);
				}
			};
		},
		"(import \"test_crate\" \"JsTest.test\" (func $test_crate.import.JsTest.test (@sym (name \
		 \"test_crate.import.JsTest.test\")) (param externref externref)))
		(import \"js_sys\" \"externref.table\" (table $js_sys.import.externref.table (@sym (name \
		 \"js_sys.externref.table\")) 2 externref))
		(func $test_crate.JsTest.test (@sym) (param $arg0_0 i32) (param $arg1_0 i32)
		  local.get $arg0_0
		  table.get $js_sys.import.externref.table (@reloc)
		  local.get $arg1_0
		  table.get $js_sys.import.externref.table (@reloc)
		  call $test_crate.import.JsTest.test (@reloc)
		)",
		"(arg0_0, arg1_0) => arg0_0.test = arg1_0",
	);
}

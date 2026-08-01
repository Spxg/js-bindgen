#![expect(
	clippy::float_cmp,
	reason = "string indices and lengths are exact integral JavaScript Number values"
)]

use js_bindgen_test::test;
use js_sys::{Array, Function, Intl, JsString, JsValue, Object, Reflect, RegExp, js_sys};

#[js_sys]
extern "js-sys" {
	#[js_sys(js_embed = "test")]
	fn js_string(value: &str) -> JsString;

	#[js_sys(js_embed = "identity")]
	fn identity(value: String) -> String;

	#[js_sys(js_embed = "identity")]
	fn optional_identity(value: Option<String>) -> Option<String>;

	#[js_sys(js_embed = "identity")]
	fn result_identity(value: String) -> Result<String, JsValue>;

	#[js_sys(js_embed = "throw")]
	fn result_error() -> Result<String, JsValue>;

	#[js_sys(js_embed = "wrong_type")]
	fn wrong_type() -> Result<String, JsValue>;

	#[js_sys(js_embed = "lone_surrogate")]
	fn lone_surrogate() -> String;

	#[js_sys(js_embed = "lone_surrogate")]
	fn lone_surrogate_js() -> JsString;

	#[js_sys(js_embed = "grow_memory")]
	fn grow_memory(value: String) -> String;
}

js_bindgen::embed_js!(module = "string", name = "test", "(value) => value");
js_bindgen::embed_js!(module = "string", name = "identity", "value => value");
js_bindgen::embed_js!(
	module = "string",
	name = "throw",
	"() => {{ throw 'error' }}"
);
js_bindgen::embed_js!(module = "string", name = "wrong_type", "() => 42");
js_bindgen::embed_js!(
	module = "string",
	name = "lone_surrogate",
	"() => '\\ud800'"
);
js_bindgen::embed_js!(
	module = "string",
	name = "grow_memory",
	"value => {{",
	#[cfg(target_arch = "wasm32")]
	"    this.#memory.grow(1)",
	#[cfg(target_arch = "wasm64")]
	"    this.#memory.grow(1n)",
	"    return value",
	"}}",
);

#[test]
fn rust_string() {
	let string = js_string("Hello, World!");
	assert_eq!(String::from(&string), "Hello, World!");
}

#[test]
fn owned_roundtrip() {
	for value in [
		"",
		"Hello, World!",
		"a\0b",
		"你好，世界！🦀",
		"\u{feff}leading byte-order mark",
	] {
		assert_eq!(identity(value.to_owned()), value);
	}
}

#[test]
fn optional_owned_roundtrip() {
	assert_eq!(optional_identity(None), None);
	assert_eq!(optional_identity(Some(String::new())), Some(String::new()));
	let value = String::from("optional 🦀");
	assert_eq!(optional_identity(Some(value.clone())), Some(value));
}

#[test]
fn result_owned_roundtrip() {
	let value = String::from("result 🦀");
	assert_eq!(result_identity(value.clone()).unwrap(), value);
	assert_eq!(
		result_error().unwrap_err(),
		JsValue::from(JsString::from("error"))
	);
	assert!(wrong_type().is_err());
}

#[test]
fn owned_conversions() {
	let string = JsString::from(String::from("owned 🦀"));
	assert_eq!(string, "owned 🦀");
	assert_eq!(String::from(string), "owned 🦀");

	let character = JsString::from('🦀');
	assert_eq!(character, "🦀");
	assert_eq!(JsString::default(), "");
	assert_eq!("parsed".parse::<JsString>().unwrap(), "parsed");
}

#[test]
fn formatting() {
	let string = JsString::from("line\n\"quoted\"");
	assert_eq!(format!("{string}"), "line\n\"quoted\"");
	assert_eq!(format!("{string:?}"), "\"line\\n\\\"quoted\\\"\"");
}

#[test]
fn replaces_lone_surrogate() {
	assert_eq!(lone_surrogate(), "\u{fffd}");
}

#[test]
fn static_methods() {
	assert_eq!(
		JsString::from_char_code(&[u32::from(b'A'), 0xd83e, 0xdd80]),
		"A🦀"
	);
	assert_eq!(JsString::from_char_code(&[0x1_0041]), "A");
	assert_eq!(
		JsString::from_code_point(&[u32::from(b'A'), '🦀' as u32]).unwrap(),
		"A🦀"
	);
	assert!(JsString::from_code_point(&[0x11_0000]).is_err());

	let raw_parts = Array::<JsString>::new_typed();
	let _ = raw_parts.push(&JsString::from(r"before\n"));
	let _ = raw_parts.push(&JsString::from("after"));
	let call_site = Object::new();
	assert!(Reflect::set_str(call_site.as_ref(), "raw", raw_parts.as_ref()).unwrap());
	let substitutions = [JsValue::from(JsString::from("value"))];
	assert_eq!(
		JsString::raw(&call_site, &substitutions).unwrap(),
		r"before\nvalueafter"
	);
}

#[test]
fn concat_locale_and_primitive_methods() {
	let value = JsString::from("one");
	assert_eq!(value.concat("two"), "onetwo");
	assert_eq!(
		value.concat_many(&[JsString::from("two"), JsString::from("three")]),
		"onetwothree"
	);

	assert!(JsString::from("a").locale_compare("b") < 0.0);
	let locales = JsValue::from(JsString::from("en-US"));
	let options = Intl::CollatorOptions::new();
	options.set_sensitivity(Intl::CollatorSensitivity::Base);
	assert_eq!(
		JsString::from("A")
			.locale_compare_with_locales_and_options("a", &locales, &options)
			.unwrap(),
		0.0
	);
	let invalid_locale = JsValue::from(JsString::from("not_a_locale"));
	assert!(
		value
			.locale_compare_with_locales("two", &invalid_locale)
			.is_err()
	);

	assert_eq!(value.to_string(), "one");
	assert_eq!(value.value_of(), "one");
	assert_eq!(
		JsString::from("I")
			.to_locale_lower_case_with_locale("tr")
			.unwrap(),
		"ı"
	);
	assert_eq!(
		JsString::from("i")
			.to_locale_upper_case_with_locale("tr")
			.unwrap(),
		"İ"
	);
	assert!(
		value
			.to_locale_lower_case_with_locales(&invalid_locale)
			.is_err()
	);
	assert!(value.normalize_with_form("invalid").is_err());
}

#[test]
fn matching_and_searching() {
	let value = JsString::from("cat1 cat22");
	let capture = RegExp::new(r"cat(\d+)").unwrap();
	let first_match = value.match_(&capture).unwrap();
	assert_eq!(first_match.length(), 2);
	assert_eq!(first_match.get(0), JsValue::from(JsString::from("cat1")));
	assert_eq!(first_match.get(1), JsValue::from(JsString::from("1")));
	assert!(value.match_(&RegExp::new("dog").unwrap()).is_none());

	let string_match = value.match_str(r"cat(\d+)").unwrap().unwrap();
	assert_eq!(string_match.get(1), Some(JsString::from("1")));
	assert!(value.match_str("[").is_err());

	let global = RegExp::new_with_flags(r"cat(\d+)", "g").unwrap();
	let global_matches = value.match_(&global).unwrap();
	assert_eq!(global_matches.length(), 2);
	assert_eq!(global_matches.get(0), JsValue::from(JsString::from("cat1")));
	assert_eq!(
		global_matches.get(1),
		JsValue::from(JsString::from("cat22"))
	);
	let all_matches: Vec<_> = value
		.match_all(&global)
		.unwrap()
		.into_iter()
		.collect::<Result<_, _>>()
		.unwrap();
	assert_eq!(all_matches.len(), 2);
	assert_eq!(all_matches[0].get(1), Some(JsString::from("1")));
	assert_eq!(all_matches[1].get(1), Some(JsString::from("22")));
	assert!(value.match_all(&capture).is_err());
	assert_eq!(
		value
			.match_all_str(r"cat(\d+)")
			.unwrap()
			.into_iter()
			.collect::<Result<Vec<_>, _>>()
			.unwrap()
			.len(),
		2
	);

	assert_eq!(value.search(&RegExp::new(r"\d{2}").unwrap()), 8.0);
	assert_eq!(value.search_str(r"\d{2}").unwrap(), 8.0);
	assert!(value.search_str("[").is_err());
}

#[test]
fn padding_replacing_and_splitting() {
	let value = JsString::from("cat cat");
	assert_eq!(JsString::from("ab").repeat(2.9).unwrap(), "abab");
	assert!(JsString::from("ab").repeat(-1.0).is_err());
	assert!(JsString::from("ab").repeat(f64::INFINITY).is_err());
	assert_eq!(JsString::from("x").pad_start(3.0), "  x");
	assert_eq!(JsString::from("x").pad_start_with_string(3.0, "0"), "00x");
	assert_eq!(JsString::from("x").pad_end(3.0), "x  ");
	assert_eq!(JsString::from("x").pad_end_with_string(3.0, "0"), "x00");

	assert_eq!(value.replace("cat", "dog"), "dog cat");
	assert_eq!(value.replace_all("cat", "dog"), "dog dog");
	let global = RegExp::new_with_flags("cat", "g").unwrap();
	assert_eq!(value.replace_regexp(&global, "dog"), "dog dog");
	assert_eq!(value.replace_all_regexp(&global, "dog").unwrap(), "dog dog");
	let non_global = RegExp::new("cat").unwrap();
	assert!(value.replace_all_regexp(&non_global, "dog").is_err());

	let upper = Function::new_with_args("value", "return value.toUpperCase()").unwrap();
	assert_eq!(
		value.replace_with_function("cat", &upper).unwrap(),
		"CAT cat"
	);
	assert_eq!(
		value.replace_regexp_with_function(&global, &upper).unwrap(),
		"CAT CAT"
	);
	assert_eq!(
		value.replace_all_with_function("cat", &upper).unwrap(),
		"CAT CAT"
	);
	assert_eq!(
		value
			.replace_all_regexp_with_function(&global, &upper)
			.unwrap(),
		"CAT CAT"
	);

	let comma_separated = JsString::from("one, two, three");
	let whole = comma_separated.split();
	assert_eq!(whole.length(), 1);
	assert_eq!(whole.get_unchecked(0), "one, two, three");
	let parts = comma_separated.split_with_separator_and_limit(", ", 2);
	assert_eq!(parts.length(), 2);
	assert_eq!(parts.get_unchecked(0), "one");
	assert_eq!(parts.get_unchecked(1), "two");
	let separator = RegExp::new(r",\s*").unwrap();
	let parts = comma_separated.split_with_regexp(&separator);
	assert_eq!(parts.length(), 3);
	assert_eq!(
		comma_separated
			.split_with_regexp_and_limit(&separator, 1)
			.length(),
		1
	);
}

#[test]
fn builtin_methods() {
	let value = JsString::from("A🦀B");
	assert_eq!(value.length(), 4.0);
	assert_eq!(value.at(-1.0).unwrap(), "B");
	assert_eq!(value.at(-1.9).unwrap(), "B");
	assert_eq!(value.char_at(0.0), "A");
	assert!((value.char_code_at(0.0) - f64::from(b'A')).abs() < f64::EPSILON);
	assert_eq!(value.code_point_at(1.0), Some('🦀' as u32));
	assert!(value.includes("🦀"));
	assert_eq!(value.index_of("B"), 3.0);
	assert_eq!(value.last_index_of("A"), 0.0);
	assert_eq!(value.slice(-1.0), "B");
	assert_eq!(value.substring_range(1.0, 3.0), "🦀");

	assert_eq!(JsString::from("  Rust 🦀  ").trim(), "Rust 🦀");
	assert_eq!(JsString::from("RuSt").to_lower_case(), "rust");
	assert_eq!(JsString::from("e\u{301}").normalize(), "é");

	let ill_formed = lone_surrogate_js();
	assert!(!ill_formed.is_well_formed());
	assert_eq!(ill_formed.to_well_formed(), "\u{fffd}");

	let characters: Vec<_> = value
		.iter()
		.map(|character| String::from(character.unwrap()))
		.collect();
	assert_eq!(characters, ["A", "🦀", "B"]);
}

#[test]
fn survives_memory_growth() {
	let value = "你好，世界！🦀".repeat(32_768);
	assert_eq!(grow_memory(value.clone()), value);
}

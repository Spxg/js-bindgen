#![expect(
	clippy::float_cmp,
	reason = "RegExp lastIndex is tested with exact integral JavaScript Number values"
)]

use js_bindgen_test::test;
use js_sys::hazard::JsCast;
use js_sys::{Array, Function, JsString, JsValue, Number, Reflect, RegExp};

#[test]
fn constructors_and_properties() {
	assert_eq!(
		RegExp::escape("foo.bar-1"),
		JsString::from("\\x66oo\\.bar\\x2d1")
	);

	let regexp = RegExp::new_with_flags("a.b", "dgimsuy").unwrap();
	assert_eq!(regexp.source(), JsString::from("a.b"));
	assert_eq!(regexp.flags(), JsString::from("dgimsuy"));
	assert!(regexp.dot_all());
	assert!(regexp.global());
	assert!(regexp.has_indices());
	assert!(regexp.ignore_case());
	assert!(regexp.multiline());
	assert!(regexp.sticky());
	assert!(regexp.unicode());
	assert!(!regexp.unicode_sets());

	let copied = RegExp::new_from_regexp(&regexp);
	assert_eq!(copied.source(), regexp.source());
	assert_eq!(copied.flags(), regexp.flags());
	let changed = RegExp::new_from_regexp_with_flags(&regexp, "i").unwrap();
	assert_eq!(changed.flags(), JsString::from("i"));

	assert!(RegExp::new("[").is_err());
	assert!(RegExp::new_with_flags("a", "gg").is_err());
	assert_eq!(regexp.to_string(), JsString::from("/a.b/dgimsuy"));
}

#[test]
fn exec_and_match_results() {
	let regexp = RegExp::new_with_flags("(?<word>a)(b)?", "dgi").unwrap();
	let result = regexp.exec("xxAb yyA").unwrap();
	assert_eq!(result.index(), 2.0);
	assert_eq!(result.input(), JsString::from("xxAb yyA"));
	assert_eq!(result.length(), 3);
	assert_eq!(result.get(0), Some(JsString::from("Ab")));
	assert_eq!(result.get(1), Some(JsString::from("A")));
	assert_eq!(result.get(2), Some(JsString::from("b")));
	assert!(result.get(3).is_none());
	assert_eq!(regexp.last_index(), 4.0);

	let groups = result.groups().unwrap();
	assert_eq!(
		Reflect::get_str(groups.as_ref(), "word").unwrap(),
		JsValue::from(JsString::from("A"))
	);
	let indices = result.indices().unwrap();
	assert_eq!(indices.length(), 3);
	let range = indices.get(0).unwrap();
	assert_eq!(range.length(), 2);
	let start: Number = range.get_unchecked(0);
	let end: Number = range.get_unchecked(1);
	assert_eq!(start.value_of(), 2.0);
	assert_eq!(end.value_of(), 4.0);
	let named = indices.groups().unwrap();
	let word = Array::<Number>::unchecked_from(Reflect::get_str(named.as_ref(), "word").unwrap());
	assert_eq!(word.length(), 2);
	assert_eq!(word.get_unchecked(0).value_of(), 2.0);
	assert_eq!(word.get_unchecked(1).value_of(), 3.0);

	let second = regexp.exec("xxAb yyA").unwrap();
	assert_eq!(second.index(), 7.0);
	assert_eq!(second.get(2), None);
	assert!(regexp.exec("xxAb yyA").is_none());
	assert_eq!(regexp.last_index(), 0.0);
	regexp.set_last_index(2.0);
	assert_eq!(regexp.last_index(), 2.0);

	let matches = regexp.match_("xxAb yyA").unwrap();
	assert_eq!(matches.length(), 2);
	let all: Vec<_> = regexp
		.match_all("xxAb yyA")
		.into_iter()
		.collect::<Result<_, _>>()
		.unwrap();
	assert_eq!(all.len(), 2);
}

#[test]
fn protocol_methods() {
	let global = RegExp::new_with_flags("a", "g").unwrap();
	assert!(global.test("aba"));
	global.set_last_index(0.0);
	assert_eq!(global.search("caba"), 1.0);
	assert_eq!(global.replace("aba", "x"), JsString::from("xbx"));

	let upper = Function::new_with_args("value", "return value.toUpperCase()").unwrap();
	assert_eq!(
		global.replace_with_function("aba", &upper).unwrap(),
		JsString::from("AbA")
	);

	let separator = RegExp::new(",").unwrap();
	let parts = separator.split("one,two,three");
	assert_eq!(parts.length(), 3);
	let limited = separator.split_with_limit("one,two,three", 2);
	assert_eq!(limited.length(), 2);
}

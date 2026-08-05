use js_bindgen_test::test;
use js_sys::Intl::{
	Collator, CollatorOptions, CollatorSensitivity, Segmenter, SegmenterGranularity,
	SegmenterOptions,
};
use js_sys::{Intl, JsString, JsValue, Number};

fn locale(value: &str) -> JsValue {
	JsString::from(value).into()
}

#[test]
fn namespace_bindings() {
	let canonical = Intl::get_canonical_locales(&locale("EN-us")).unwrap();
	assert_eq!(canonical.length(), 1);
	assert_eq!(canonical.get_unchecked(0), "en-US");

	assert!(Intl::supported_values_of("calendar").is_ok());
	assert!(Intl::supported_values_of("not-an-intl-key").is_err());
}

#[test]
fn constructor_options_and_function_property() {
	let options = CollatorOptions::new();
	options.set_sensitivity(CollatorSensitivity::Base);
	options.set_numeric(true);
	assert_eq!(options.sensitivity(), Some(CollatorSensitivity::Base));
	assert_eq!(options.numeric(), Some(true));

	let collator = Collator::new_with_locales_and_options(&locale("en-US"), &options).unwrap();
	let ordering = collator
		.compare()
		.call(
			&JsValue::UNDEFINED,
			&[
				JsValue::from(JsString::from("2")),
				JsValue::from(JsString::from("10")),
			],
		)
		.unwrap();
	assert!(Number::new(&ordering).unwrap().value_of() < 0.0);

	let resolved = collator.resolved_options();
	assert_eq!(resolved.sensitivity(), "base");
	assert_eq!(resolved.numeric(), Some(true));
}

#[test]
fn typed_iterator_result() {
	let options = SegmenterOptions::new();
	options.set_granularity(SegmenterGranularity::Word);
	let segmenter = Segmenter::new_with_locales_and_options(&locale("en-US"), &options).unwrap();

	let segments = segmenter.segment("Hello, world!");
	let first = segments.iter().next().unwrap().unwrap();
	assert_eq!(first.segment(), "Hello");
	assert_eq!(first.is_word_like(), Some(true));

	assert!(Segmenter::new_with_locales(&locale("not_a_locale")).is_err());
}

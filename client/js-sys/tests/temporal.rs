#![expect(
	clippy::float_cmp,
	reason = "Temporal epoch values and duration fields are exact integer-valued numbers"
)]

use js_bindgen_test::test;
use js_sys::{JsString, JsValue, Temporal, js_sys};

js_bindgen::embed_js!(
	module = "temporal",
	name = "supported",
	"() => typeof globalThis.Temporal === 'object'",
);

#[js_sys]
extern "js-sys" {
	#[js_sys(js_embed = "supported")]
	fn temporal_supported() -> bool;
}

fn string(value: &str) -> JsValue {
	JsString::from(value).into()
}

#[test]
fn duration_and_instant() {
	if !temporal_supported() {
		return;
	}

	let duration = Temporal::Duration::from(&string("P1DT2H3M4.005006007S")).unwrap();
	assert_eq!(duration.days(), 1.0);

	let instant = Temporal::Instant::from_epoch_milliseconds(1_700_000_000_000.0).unwrap();
	assert_eq!(instant.epoch_milliseconds(), 1_700_000_000_000.0);
	let date = Temporal::PlainDate::new(2024, 2, 29).unwrap();
	assert_eq!((date.year(), date.month(), date.day()), (2024, 2, 29));
	assert!(Temporal::Duration::from(&string("invalid")).is_err());
}

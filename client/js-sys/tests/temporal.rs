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
	assert_eq!(duration.hours(), 2.0);
	assert_eq!(duration.minutes(), 3.0);
	assert_eq!(duration.seconds(), 4.0);
	assert_eq!(duration.milliseconds(), 5.0);
	assert_eq!(duration.microseconds(), 6.0);
	assert_eq!(duration.nanoseconds(), 7.0);
	assert_eq!(duration.sign(), 1);
	assert!(!duration.blank());
	assert_eq!(duration.to_string(), "P1DT2H3M4.005006007S");
	assert_eq!(duration.negated().sign(), -1);

	let instant = Temporal::Instant::from_epoch_milliseconds(1_700_000_000_000.0).unwrap();
	assert_eq!(instant.epoch_milliseconds(), 1_700_000_000_000.0);
	assert_eq!(instant.to_string(), "2023-11-14T22:13:20Z");
}

#[test]
fn plain_date_time_types() {
	if !temporal_supported() {
		return;
	}

	let date = Temporal::PlainDate::new(2024, 2, 29).unwrap();
	assert_eq!((date.year(), date.month(), date.day()), (2024, 2, 29));
	assert!(date.in_leap_year());
	assert_eq!(date.to_string(), "2024-02-29");

	let time = Temporal::PlainTime::new_with_values(23, 59, 58, 987, 654, 321).unwrap();
	assert_eq!(
		(
			time.hour(),
			time.minute(),
			time.second(),
			time.millisecond(),
			time.microsecond(),
			time.nanosecond(),
		),
		(23, 59, 58, 987, 654, 321),
	);

	let date_time =
		Temporal::PlainDateTime::from(&string("2024-02-29T23:59:58.987654321")).unwrap();
	assert_eq!(date_time.to_plain_date().to_string(), date.to_string());
	assert_eq!(date_time.to_plain_time().to_string(), time.to_string());
	assert_eq!(date_time.to_string(), "2024-02-29T23:59:58.987654321");
}

#[test]
fn calendar_and_zoned_types() {
	if !temporal_supported() {
		return;
	}

	let month_day = Temporal::PlainMonthDay::new(2, 29).unwrap();
	assert_eq!(month_day.month_code(), "M02");
	assert_eq!(month_day.day(), 29);

	let year_month = Temporal::PlainYearMonth::new(2024, 2).unwrap();
	assert_eq!((year_month.year(), year_month.month()), (2024, 2));
	assert_eq!(year_month.days_in_month(), 29);
	assert!(year_month.in_leap_year());

	let instant = Temporal::Instant::from_epoch_milliseconds(1_700_000_000_000.0).unwrap();
	let zoned = instant.to_zoned_date_time_iso(&string("UTC")).unwrap();
	assert_eq!(zoned.time_zone_id(), "UTC");
	assert_eq!(zoned.epoch_milliseconds(), instant.epoch_milliseconds());
	assert_eq!(zoned.hours_in_day().unwrap(), 24.0);
	assert_eq!(zoned.start_of_day().unwrap().hour(), 0);
	assert_eq!(zoned.with_plain_time().unwrap().hour(), 0);
	assert_eq!(
		zoned.to_instant().epoch_milliseconds(),
		instant.epoch_milliseconds()
	);
}

#[test]
fn invalid_inputs_are_reported() {
	if !temporal_supported() {
		return;
	}

	let invalid = string("not a Temporal value");
	assert!(Temporal::Duration::from(&invalid).is_err());
	assert!(Temporal::Instant::from(&invalid).is_err());
	assert!(Temporal::PlainDate::from(&invalid).is_err());
	assert!(Temporal::PlainTime::from(&invalid).is_err());
	assert!(Temporal::PlainDateTime::from(&invalid).is_err());
	assert!(Temporal::PlainMonthDay::from(&invalid).is_err());
	assert!(Temporal::PlainYearMonth::from(&invalid).is_err());
	assert!(Temporal::ZonedDateTime::from(&invalid).is_err());
}

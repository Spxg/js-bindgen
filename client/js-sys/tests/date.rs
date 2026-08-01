#![expect(
	clippy::float_cmp,
	reason = "the tested Date fields are exact integral JavaScript Number values"
)]

use js_bindgen_test::test;
use js_sys::{Date, JsString, JsValue, Symbol};

#[test]
fn constructors_and_static_methods() {
	let epoch = Date::new_with_milliseconds(0.0);
	assert_eq!(epoch.get_time(), 0.0);
	assert_eq!(epoch.value_of(), 0.0);
	assert_eq!(epoch.get_utc_full_year(), 1970.0);
	assert_eq!(epoch.get_utc_month(), 0.0);
	assert_eq!(epoch.get_utc_date(), 1.0);
	assert_eq!(epoch.get_utc_day(), 4.0);
	assert_eq!(epoch.get_utc_hours(), 0.0);
	assert_eq!(epoch.get_utc_minutes(), 0.0);
	assert_eq!(epoch.get_utc_seconds(), 0.0);
	assert_eq!(epoch.get_utc_milliseconds(), 0.0);

	let parsed = Date::new_with_string("2020-02-03T04:05:06.007Z");
	assert_eq!(parsed.get_time(), Date::parse("2020-02-03T04:05:06.007Z"));
	assert_eq!(parsed.get_utc_full_year(), 2020.0);
	assert_eq!(parsed.get_utc_month(), 1.0);
	assert_eq!(parsed.get_utc_date(), 3.0);

	let copied = Date::new_with_date(&parsed);
	assert_eq!(copied.get_time(), parsed.get_time());
	let local = Date::new_with_year_month_day_hour_minute_second_millisecond(
		2021.0, 5.0, 7.0, 8.0, 9.0, 10.0, 11.0,
	);
	assert_eq!(local.get_full_year(), 2021.0);
	assert_eq!(local.get_month(), 5.0);
	assert_eq!(local.get_date(), 7.0);
	assert_eq!(local.get_hours(), 8.0);
	assert_eq!(local.get_minutes(), 9.0);
	assert_eq!(local.get_seconds(), 10.0);
	assert_eq!(local.get_milliseconds(), 11.0);

	let timestamp =
		Date::utc_with_day_hour_minute_second_millisecond(2020.0, 1.0, 3.0, 4.0, 5.0, 6.0, 7.0);
	assert_eq!(timestamp, parsed.get_time());
	assert_eq!(Date::utc(1970.0, 0.0), 0.0);
	assert!(Date::now().is_finite());
	assert!(Date::default().get_time().is_finite());
	assert!(Date::new().get_time().is_finite());

	let symbol = Symbol::new();
	assert!(Date::new_with_value(symbol.as_ref()).is_err());
}

#[test]
fn setters_and_strings() {
	let date = Date::new_with_milliseconds(0.0);
	let _ = date.set_utc_full_year_with_month_date(2024.0, 1.0, 29.0);
	let _ = date.set_utc_hours_with_minutes_seconds_milliseconds(12.0, 34.0, 56.0, 789.0);
	assert_eq!(date.get_utc_full_year(), 2024.0);
	assert_eq!(date.get_utc_month(), 1.0);
	assert_eq!(date.get_utc_date(), 29.0);
	assert_eq!(date.get_utc_hours(), 12.0);
	assert_eq!(date.get_utc_minutes(), 34.0);
	assert_eq!(date.get_utc_seconds(), 56.0);
	assert_eq!(date.get_utc_milliseconds(), 789.0);

	let _ = date.set_utc_month_with_date(2.0, 1.0);
	let _ = date.set_utc_minutes_with_seconds_milliseconds(2.0, 3.0, 4.0);
	let _ = date.set_utc_seconds_with_milliseconds(5.0, 6.0);
	assert_eq!(date.get_utc_month(), 2.0);
	assert_eq!(date.get_utc_date(), 1.0);
	assert_eq!(date.get_utc_minutes(), 2.0);
	assert_eq!(date.get_utc_seconds(), 5.0);
	assert_eq!(date.get_utc_milliseconds(), 6.0);

	let iso = date.to_iso_string().unwrap();
	assert_eq!(date.to_json(), Some(iso.clone()));
	assert!(!String::from(&date.to_date_string()).is_empty());
	assert!(!String::from(&date.to_time_string()).is_empty());
	assert!(!String::from(&date.to_utc_string()).is_empty());
	assert!(!String::from(&date.to_string()).is_empty());
	assert!(!String::from(&date.to_locale_date_string().unwrap()).is_empty());
	assert!(!String::from(&date.to_locale_string().unwrap()).is_empty());
	assert!(!String::from(&date.to_locale_time_string().unwrap()).is_empty());

	let locale = JsValue::from(JsString::from("en-US"));
	assert!(date.to_locale_string_with_locales(&locale).is_ok());

	let _ = date.set_time(f64::NAN);
	assert!(date.get_time().is_nan());
	assert!(date.get_full_year().is_nan());
	assert!(date.to_iso_string().is_err());
	assert!(date.to_json().is_none());
}

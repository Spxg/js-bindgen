#![expect(
	clippy::float_cmp,
	reason = "the tested Date fields are exact integral JavaScript Number values"
)]

use js_bindgen_test::test;
use js_sys::{Date, Symbol};

#[test]
fn constructors_static_and_instance_methods() {
	let epoch = Date::new_with_milliseconds(0.0);
	assert_eq!(epoch.get_time(), 0.0);
	assert_eq!(epoch.get_utc_full_year(), 1970.0);
	assert_eq!(Date::utc(1970.0, 0.0), 0.0);
	let _ = epoch.set_utc_full_year_with_month_date(2024.0, 1.0, 29.0);
	assert_eq!(epoch.get_utc_full_year(), 2024.0);
	assert!(!String::from(&epoch.to_iso_string().unwrap()).is_empty());
	assert!(Date::new_with_value(Symbol::new().as_ref()).is_err());
}

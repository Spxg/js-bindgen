use js_bindgen_test::test;
use js_sys::Intl::{
	Collator, CollatorCaseFirst, CollatorOptions, CollatorSensitivity, CollatorUsage,
	DateTimeFormat, DateTimeFormatFractionalSecondDigits, DateTimeFormatMatcher,
	DateTimeFormatMonthStyle, DateTimeFormatNumericStyle, DateTimeFormatOptions,
	DateTimeFormatPartType, DateTimeFormatRangeSource, DateTimeFormatStyle,
	DateTimeFormatTextStyle, DateTimeFormatTimeZoneName, DisplayNames, DisplayNamesFallback,
	DisplayNamesOptions, DisplayNamesStyle, DisplayNamesType, Duration, DurationFormat,
	DurationFormatOptions, DurationFormatPartType, DurationFormatStyle, DurationSubsecondUnitStyle,
	DurationTimeUnitStyle, DurationUnit, DurationUnitDisplay, DurationUnitStyle, FirstDayOfWeek,
	HourCycle, ListFormat, ListFormatOptions, ListFormatStyle, ListFormatType, Locale,
	LocaleMatcher, LocaleMatcherOptions, LocaleOptions, NumberFormat, NumberFormatOptions,
	NumberFormatPartType, NumberFormatRoundingIncrement, NumberFormatRoundingMode,
	NumberFormatRoundingPriority, NumberFormatSignDisplay, NumberFormatStyle,
	NumberFormatTrailingZeroDisplay, NumberFormatUseGrouping, PluralRules, PluralRulesOptions,
	PluralRulesRoundingIncrement, PluralRulesRoundingMode, PluralRulesRoundingPriority,
	PluralRulesTrailingZeroDisplay, PluralRulesType, RelativeTimeFormat, RelativeTimeFormatNumeric,
	RelativeTimeFormatOptions, RelativeTimeFormatStyle, RelativeTimeUnit, Segmenter,
	SegmenterGranularity, SegmenterOptions, TextDirection,
};
use js_sys::hazard::JsCast;
use js_sys::{Intl, JsString, JsValue, Number, Object, Reflect};

fn locale(value: &str) -> JsValue {
	JsString::from(value).into()
}

#[test]
fn namespace_functions() {
	let canonical = Intl::get_canonical_locales(&locale("EN-us")).unwrap();
	assert_eq!(canonical.length(), 1);
	assert_eq!(canonical.get_unchecked(0), "en-US");

	let calendars = Intl::supported_values_of("calendar").unwrap();
	assert!(calendars.includes(&JsString::from("gregory")));
	assert!(Intl::supported_values_of("not-an-intl-key").is_err());
}

#[test]
fn collator_numeric_comparison_and_options() {
	let options = CollatorOptions::new();
	options.set_usage(CollatorUsage::Sort);
	options.set_sensitivity(CollatorSensitivity::Base);
	options.set_numeric(true);
	options.set_case_first(CollatorCaseFirst::Upper);
	assert_eq!(options.usage(), Some(CollatorUsage::Sort));
	assert_eq!(options.sensitivity(), Some(CollatorSensitivity::Base));
	assert_eq!(options.numeric(), Some(true));
	assert_eq!(options.case_first(), Some(CollatorCaseFirst::Upper));

	let collator = Collator::new_with_locales_and_options(&locale("en-US"), &options).unwrap();
	let arguments = [
		JsValue::from(JsString::from("2")),
		JsValue::from(JsString::from("10")),
	];
	let ordering = collator
		.compare()
		.call(&JsValue::UNDEFINED, &arguments)
		.unwrap();
	assert!(Number::new(&ordering).unwrap().value_of() < 0.0);

	let resolved = collator.resolved_options();
	assert_eq!(resolved.locale(), "en-US");
	assert_eq!(resolved.usage(), "sort");
	assert_eq!(resolved.sensitivity(), "base");
	assert_eq!(resolved.numeric(), Some(true));
}

#[test]
#[expect(
	clippy::float_cmp,
	reason = "segment indices are exact integral JavaScript Number values"
)]
fn segmenter_word_segments() {
	let options = SegmenterOptions::new();
	options.set_granularity(SegmenterGranularity::Word);
	assert_eq!(options.granularity(), Some(SegmenterGranularity::Word));
	let segmenter = Segmenter::new_with_locales_and_options(&locale("en-US"), &options).unwrap();

	let resolved = segmenter.resolved_options();
	assert_eq!(resolved.locale(), "en-US");
	assert_eq!(resolved.granularity(), "word");

	let input = "Hello, world!";
	let segments = segmenter.segment(input);
	let containing = segments.containing_at(8.0).unwrap();
	assert_eq!(containing.segment(), "world");
	assert_eq!(containing.index(), 7.0);
	assert_eq!(containing.input(), input);
	assert_eq!(containing.is_word_like(), Some(true));
	assert_eq!(segments.containing().unwrap().segment(), "Hello");
	assert_eq!(segments.containing_at(f64::NAN).unwrap().segment(), "Hello");
	assert!(segments.containing_at(13.0).is_none());

	let values = segments.iter().collect::<Result<Vec<_>, _>>().unwrap();
	let actual: Vec<_> = values
		.iter()
		.map(|value| {
			(
				String::from(value.segment()),
				value.index(),
				value.is_word_like(),
			)
		})
		.collect();
	assert_eq!(
		actual,
		[
			(String::from("Hello"), 0.0, Some(true)),
			(String::from(","), 5.0, Some(false)),
			(String::from(" "), 6.0, Some(false)),
			(String::from("world"), 7.0, Some(true)),
			(String::from("!"), 12.0, Some(false)),
		]
	);
}

#[test]
fn list_format_options_and_parts() {
	let options = ListFormatOptions::new();
	options.set_type(ListFormatType::Unit);
	options.set_style(ListFormatStyle::Narrow);
	assert_eq!(options.type_(), Some(ListFormatType::Unit));
	assert_eq!(options.style(), Some(ListFormatStyle::Narrow));
	let list_format = ListFormat::new_with_locales_and_options(&locale("en-US"), &options).unwrap();

	let resolved = list_format.resolved_options();
	assert_eq!(resolved.locale(), "en-US");
	assert_eq!(resolved.type_(), "unit");
	assert_eq!(resolved.style(), "narrow");

	let expected = ["1 meter", "2 liters", "3 seconds"];
	let list = js_sys::Array::new_typed();
	for value in expected {
		let _ = list.push(&JsString::from(value));
	}

	let formatted = String::from(list_format.format(list.as_ref()).unwrap());
	let parts = list_format.format_to_parts(list.as_ref()).unwrap();
	let mut reconstructed = String::new();
	let mut elements = Vec::new();
	for index in 0..parts.length() {
		let part = parts.get_unchecked(index);
		let part_type = String::from(part.type_());
		let value = String::from(part.value());
		reconstructed.push_str(&value);
		match part_type.as_str() {
			"element" => elements.push(value),
			"literal" => assert!(!value.is_empty()),
			other => panic!("unexpected list part type: {other}"),
		}
	}
	assert_eq!(reconstructed, formatted);
	assert_eq!(elements, expected);

	let not_iterable = Object::new();
	assert!(list_format.format(not_iterable.as_ref()).is_err());
	assert!(list_format.format_to_parts(not_iterable.as_ref()).is_err());
}

#[test]
fn display_names_options_and_fallback() {
	let options = DisplayNamesOptions::new(DisplayNamesType::Currency);
	options.set_style(DisplayNamesStyle::Short);
	options.set_fallback(DisplayNamesFallback::None);
	assert_eq!(
		options.display_names_type(),
		Some(DisplayNamesType::Currency)
	);
	assert_eq!(options.style(), Some(DisplayNamesStyle::Short));
	assert_eq!(options.fallback(), Some(DisplayNamesFallback::None));
	let display_names = DisplayNames::new(&locale("en-US"), &options).unwrap();

	let resolved = display_names.resolved_options();
	assert_eq!(resolved.locale(), "en-US");
	assert_eq!(resolved.style(), "short");
	assert_eq!(resolved.display_names_type(), "currency");
	assert_eq!(resolved.fallback(), "none");
	assert!(resolved.language_display().is_none());

	let known = display_names.of("USD").unwrap().unwrap();
	assert!(!String::from(known).is_empty());
	assert!(display_names.of("QQQ").unwrap().is_none());
	assert!(display_names.of("US").is_err());
}

#[test]
fn locale_options_and_information() {
	let matcher = LocaleMatcherOptions::new();
	matcher.set_locale_matcher(LocaleMatcher::Lookup);
	assert_eq!(matcher.locale_matcher(), Some(LocaleMatcher::Lookup));
	assert!(Locale::new("not_a_locale").is_err());

	let options = LocaleOptions::new();
	options.set_region("US");
	options.set_hour_cycle(HourCycle::H12);
	options.set_case_first(CollatorCaseFirst::Upper);
	options.set_numeric(true);
	options.set_first_day_of_week(FirstDayOfWeek::Monday);
	assert_eq!(options.region(), Some(JsString::from("US")));
	assert_eq!(options.hour_cycle(), Some(HourCycle::H12));
	assert_eq!(options.case_first(), Some(CollatorCaseFirst::Upper));
	assert_eq!(options.numeric(), Some(true));
	assert_eq!(options.first_day_of_week(), Some(FirstDayOfWeek::Monday));

	let locale = Locale::new_with_options("en", &options).unwrap();
	assert_eq!(locale.base_name(), "en-US");
	assert_eq!(locale.language(), "en");
	assert_eq!(locale.region(), Some(JsString::from("US")));
	assert_eq!(locale.hour_cycle(), Some(HourCycle::H12));
	assert_eq!(locale.case_first(), Some(CollatorCaseFirst::Upper));
	assert!(locale.numeric());
	if let Some(first_day_of_week) = locale.first_day_of_week() {
		assert_eq!(first_day_of_week, FirstDayOfWeek::Monday);
	}
	if Reflect::has_str(locale.as_ref(), "getCalendars").unwrap() {
		assert_ne!(locale.get_calendars().length(), 0);
	}
	if Reflect::has_str(locale.as_ref(), "getNumberingSystems").unwrap() {
		assert_ne!(locale.get_numbering_systems().length(), 0);
	}
	if Reflect::has_str(locale.as_ref(), "getTextInfo").unwrap() {
		assert_eq!(
			locale.get_text_info().unwrap().direction(),
			Some(TextDirection::LeftToRight)
		);
	}
	if Reflect::has_str(locale.as_ref(), "getWeekInfo").unwrap() {
		let week = locale.get_week_info().unwrap();
		assert!((1..=7).contains(&week.first_day()));
		if let Some(minimal_days) = week.minimal_days() {
			assert!((1..=7).contains(&minimal_days));
		}
		assert_ne!(week.weekend().length(), 0);
	}
	assert_eq!(locale.minimize().maximize().language(), "en");
	assert!(locale.to_string().starts_with("en"));
}

#[test]
fn date_time_format_options_parts_and_range() {
	let options = DateTimeFormatOptions::new();
	options.set_locale_matcher(LocaleMatcher::Lookup);
	options.set_calendar("gregory");
	options.set_numbering_system("latn");
	options.set_hour_12(false);
	options.set_hour_cycle(HourCycle::H23);
	options.set_time_zone("UTC");
	options.set_weekday(DateTimeFormatTextStyle::Short);
	options.set_era(DateTimeFormatTextStyle::Short);
	options.set_year(DateTimeFormatNumericStyle::Numeric);
	options.set_month(DateTimeFormatMonthStyle::TwoDigit);
	options.set_day(DateTimeFormatNumericStyle::TwoDigit);
	options.set_day_period(DateTimeFormatTextStyle::Narrow);
	options.set_hour(DateTimeFormatNumericStyle::TwoDigit);
	options.set_minute(DateTimeFormatNumericStyle::TwoDigit);
	options.set_second(DateTimeFormatNumericStyle::TwoDigit);
	options.set_fractional_second_digits(DateTimeFormatFractionalSecondDigits::Three);
	options.set_time_zone_name(DateTimeFormatTimeZoneName::Short);
	options.set_format_matcher(DateTimeFormatMatcher::Basic);
	assert_eq!(options.locale_matcher(), Some(LocaleMatcher::Lookup));
	assert_eq!(options.calendar(), Some(JsString::from("gregory")));
	assert_eq!(options.numbering_system(), Some(JsString::from("latn")));
	assert_eq!(options.hour_12(), Some(false));
	assert_eq!(options.hour_cycle(), Some(HourCycle::H23));
	assert_eq!(options.time_zone(), Some(JsString::from("UTC")));
	assert_eq!(options.weekday(), Some(DateTimeFormatTextStyle::Short));
	assert_eq!(options.era(), Some(DateTimeFormatTextStyle::Short));
	assert_eq!(options.year(), Some(DateTimeFormatNumericStyle::Numeric));
	assert_eq!(options.month(), Some(DateTimeFormatMonthStyle::TwoDigit));
	assert_eq!(options.day(), Some(DateTimeFormatNumericStyle::TwoDigit));
	assert_eq!(options.day_period(), Some(DateTimeFormatTextStyle::Narrow));
	assert_eq!(options.hour(), Some(DateTimeFormatNumericStyle::TwoDigit));
	assert_eq!(options.minute(), Some(DateTimeFormatNumericStyle::TwoDigit));
	assert_eq!(options.second(), Some(DateTimeFormatNumericStyle::TwoDigit));
	assert_eq!(
		options.fractional_second_digits(),
		Some(DateTimeFormatFractionalSecondDigits::Three)
	);
	assert_eq!(
		options.time_zone_name(),
		Some(DateTimeFormatTimeZoneName::Short)
	);
	assert_eq!(options.format_matcher(), Some(DateTimeFormatMatcher::Basic));

	let format = DateTimeFormat::new_with_locales_and_options(&locale("en-US"), &options).unwrap();
	let resolved = format.resolved_options();
	assert!(resolved.locale().starts_with("en"));
	assert_eq!(resolved.calendar(), "gregory");
	assert_eq!(resolved.numbering_system(), "latn");
	assert_eq!(resolved.time_zone(), "UTC");
	assert_eq!(resolved.hour_cycle(), Some(HourCycle::H23));
	assert_eq!(resolved.hour_12(), Some(false));
	assert_eq!(resolved.weekday(), Some(DateTimeFormatTextStyle::Short));
	assert_eq!(resolved.era(), Some(DateTimeFormatTextStyle::Short));
	assert_eq!(resolved.year(), Some(DateTimeFormatNumericStyle::Numeric));
	assert_eq!(resolved.month(), Some(DateTimeFormatMonthStyle::TwoDigit));
	assert_eq!(resolved.day(), Some(DateTimeFormatNumericStyle::TwoDigit));
	assert_eq!(resolved.hour(), Some(DateTimeFormatNumericStyle::TwoDigit));
	assert_eq!(
		resolved.minute(),
		Some(DateTimeFormatNumericStyle::TwoDigit)
	);
	assert_eq!(
		resolved.second(),
		Some(DateTimeFormatNumericStyle::TwoDigit)
	);
	assert_eq!(
		resolved.fractional_second_digits(),
		Some(DateTimeFormatFractionalSecondDigits::Three)
	);
	assert_eq!(
		resolved.time_zone_name(),
		Some(DateTimeFormatTimeZoneName::Short)
	);

	let start: JsValue = Number::new(&locale("1577934245678")).unwrap().into();
	let end: JsValue = Number::new(&locale("1577937845678")).unwrap().into();
	let formatted = JsString::unchecked_from(
		format
			.format()
			.call(&JsValue::UNDEFINED, core::slice::from_ref(&start))
			.unwrap(),
	);
	let parts = format.format_to_parts_with_date(&start).unwrap();
	let mut reconstructed = String::new();
	let mut saw_year = false;
	let mut saw_fractional_second = false;
	for index in 0..parts.length() {
		let part = parts.get_unchecked(index);
		let value = String::from(part.value());
		reconstructed.push_str(&value);
		match part.type_().unwrap() {
			DateTimeFormatPartType::Year => {
				saw_year = true;
				assert_eq!(value, "2020");
			}
			DateTimeFormatPartType::FractionalSecond => {
				saw_fractional_second = true;
				assert_eq!(value, "678");
			}
			_ => {}
		}
	}
	assert_eq!(formatted, reconstructed);
	assert!(saw_year && saw_fractional_second);

	let range = format.format_range(&start, &end).unwrap();
	let range_parts = format.format_range_to_parts(&start, &end).unwrap();
	let mut reconstructed = String::new();
	let mut saw_start = false;
	let mut saw_end = false;
	let mut saw_shared = false;
	for index in 0..range_parts.length() {
		let part = range_parts.get_unchecked(index);
		assert!(part.type_().is_some());
		reconstructed.push_str(&String::from(part.value()));
		match part.source().unwrap() {
			DateTimeFormatRangeSource::StartRange => saw_start = true,
			DateTimeFormatRangeSource::EndRange => saw_end = true,
			DateTimeFormatRangeSource::Shared => saw_shared = true,
		}
	}
	assert_eq!(range, reconstructed);
	assert!(saw_start && saw_end && saw_shared);

	let style_options = DateTimeFormatOptions::new();
	style_options.set_time_zone("UTC");
	style_options.set_date_style(DateTimeFormatStyle::Full);
	style_options.set_time_style(DateTimeFormatStyle::Long);
	assert_eq!(style_options.date_style(), Some(DateTimeFormatStyle::Full));
	assert_eq!(style_options.time_style(), Some(DateTimeFormatStyle::Long));
	let styled = DateTimeFormat::new_with_locales_and_options(&locale("en-US"), &style_options)
		.unwrap()
		.resolved_options();
	assert_eq!(styled.date_style(), Some(DateTimeFormatStyle::Full));
	assert_eq!(styled.time_style(), Some(DateTimeFormatStyle::Long));
}

#[test]
fn number_format_options_parts_and_range() {
	let options = NumberFormatOptions::new();
	options.set_locale_matcher(LocaleMatcher::Lookup);
	options.set_style(NumberFormatStyle::Decimal);
	options.set_minimum_fraction_digits(2);
	options.set_maximum_fraction_digits(2);
	options.set_rounding_priority(NumberFormatRoundingPriority::Auto);
	options.set_rounding_increment(NumberFormatRoundingIncrement::One);
	options.set_rounding_mode(NumberFormatRoundingMode::HalfEven);
	options.set_trailing_zero_display(NumberFormatTrailingZeroDisplay::Auto);
	options.set_use_grouping(NumberFormatUseGrouping::False);
	options.set_sign_display(NumberFormatSignDisplay::Auto);
	assert_eq!(options.locale_matcher(), Some(LocaleMatcher::Lookup));
	assert_eq!(options.style(), Some(NumberFormatStyle::Decimal));
	assert_eq!(options.minimum_fraction_digits(), Some(2));
	assert_eq!(options.maximum_fraction_digits(), Some(2));
	assert_eq!(
		options.rounding_priority(),
		Some(NumberFormatRoundingPriority::Auto)
	);
	assert_eq!(
		options.rounding_increment(),
		Some(NumberFormatRoundingIncrement::One)
	);
	assert_eq!(
		options.rounding_mode(),
		Some(NumberFormatRoundingMode::HalfEven)
	);
	assert_eq!(
		options.trailing_zero_display(),
		Some(NumberFormatTrailingZeroDisplay::Auto)
	);
	assert_eq!(options.use_grouping(), Some(NumberFormatUseGrouping::False));
	assert_eq!(options.sign_display(), Some(NumberFormatSignDisplay::Auto));

	let number_format =
		NumberFormat::new_with_locales_and_options(&locale("en-US"), &options).unwrap();
	let resolved = number_format.resolved_options();
	assert!(resolved.locale().starts_with("en"));
	assert_eq!(resolved.style(), Some(NumberFormatStyle::Decimal));
	assert_eq!(resolved.minimum_fraction_digits(), Some(2));
	assert_eq!(resolved.maximum_fraction_digits(), Some(2));
	assert_eq!(
		resolved.rounding_mode(),
		Some(NumberFormatRoundingMode::HalfEven)
	);
	assert_eq!(
		resolved.use_grouping(),
		Some(NumberFormatUseGrouping::False)
	);

	let value = locale("1234.5");
	let formatted = JsString::unchecked_from(
		number_format
			.format()
			.call(&JsValue::UNDEFINED, core::slice::from_ref(&value))
			.unwrap(),
	);
	assert_eq!(formatted, "1234.50");

	let parts = number_format.format_to_parts_with_value(&value).unwrap();
	let mut reconstructed = String::new();
	let mut saw_integer = false;
	let mut saw_fraction = false;
	for index in 0..parts.length() {
		let part = parts.get_unchecked(index);
		reconstructed.push_str(&String::from(part.value()));
		match part.type_().unwrap() {
			NumberFormatPartType::Integer => saw_integer = true,
			NumberFormatPartType::Fraction => saw_fraction = true,
			_ => {}
		}
	}
	assert_eq!(formatted, reconstructed);
	assert!(saw_integer && saw_fraction);

	let start = locale("1");
	let end = locale("2");
	let range = number_format.format_range(&start, &end).unwrap();
	let range_parts = number_format.format_range_to_parts(&start, &end).unwrap();
	let mut reconstructed = String::new();
	for index in 0..range_parts.length() {
		let part = range_parts.get_unchecked(index);
		assert!(part.type_().is_some());
		assert!(part.source().is_some());
		reconstructed.push_str(&String::from(part.value()));
	}
	assert_eq!(range, reconstructed);

	let approximate_options = NumberFormatOptions::new();
	approximate_options.set_maximum_fraction_digits(0);
	let approximate =
		NumberFormat::new_with_locales_and_options(&locale("en"), &approximate_options).unwrap();
	let parts = approximate
		.format_range_to_parts(&locale("2.9"), &locale("3.1"))
		.unwrap();
	assert!((0..parts.length()).any(|index| {
		parts.get_unchecked(index).type_() == Some(NumberFormatPartType::ApproximatelySign)
	}));
}

#[test]
fn plural_rules_options_selection_and_range() {
	let options = PluralRulesOptions::new();
	options.set_locale_matcher(LocaleMatcher::Lookup);
	options.set_type(PluralRulesType::Ordinal);
	options.set_minimum_integer_digits(1);
	options.set_rounding_priority(PluralRulesRoundingPriority::Auto);
	options.set_rounding_increment(PluralRulesRoundingIncrement::One);
	options.set_rounding_mode(PluralRulesRoundingMode::HalfExpand);
	options.set_trailing_zero_display(PluralRulesTrailingZeroDisplay::Auto);
	assert_eq!(options.locale_matcher(), Some(LocaleMatcher::Lookup));
	assert_eq!(options.type_(), Some(PluralRulesType::Ordinal));
	assert_eq!(options.minimum_integer_digits(), Some(1));
	assert_eq!(
		options.rounding_priority(),
		Some(PluralRulesRoundingPriority::Auto)
	);
	assert_eq!(
		options.rounding_increment(),
		Some(PluralRulesRoundingIncrement::One)
	);
	assert_eq!(
		options.rounding_mode(),
		Some(PluralRulesRoundingMode::HalfExpand)
	);
	assert_eq!(
		options.trailing_zero_display(),
		Some(PluralRulesTrailingZeroDisplay::Auto)
	);

	let rules = PluralRules::new_with_locales_and_options(&locale("en-US"), &options).unwrap();
	let resolved = rules.resolved_options();
	assert!(resolved.locale().starts_with("en"));
	assert_eq!(resolved.type_(), "ordinal");
	assert_eq!(resolved.minimum_integer_digits(), Some(1));
	assert!(
		resolved
			.plural_categories()
			.includes(&JsString::from("one"))
	);
	assert_eq!(rules.select(1.0), "one");
	assert_eq!(rules.select(2.0), "two");
	assert_eq!(rules.select(3.0), "few");
	assert_eq!(rules.select(4.0), "other");
	assert_eq!(rules.select_range(1.0, 2.0).unwrap(), "other");
}

#[test]
fn relative_time_format_options_and_parts() {
	let options = RelativeTimeFormatOptions::new();
	options.set_locale_matcher(LocaleMatcher::Lookup);
	options.set_numbering_system("latn");
	options.set_style(RelativeTimeFormatStyle::Short);
	options.set_numeric(RelativeTimeFormatNumeric::Auto);
	assert_eq!(options.locale_matcher(), Some(LocaleMatcher::Lookup));
	assert_eq!(options.numbering_system(), Some(JsString::from("latn")));
	assert_eq!(options.style(), Some(RelativeTimeFormatStyle::Short));
	assert_eq!(options.numeric(), Some(RelativeTimeFormatNumeric::Auto));

	let format =
		RelativeTimeFormat::new_with_locales_and_options(&locale("en-US"), &options).unwrap();
	let resolved = format.resolved_options();
	assert!(resolved.locale().starts_with("en"));
	assert_eq!(resolved.style(), "short");
	assert_eq!(resolved.numeric(), "auto");
	assert_eq!(resolved.numbering_system(), "latn");
	assert_eq!(
		format.format(-1.0, RelativeTimeUnit::Day).unwrap(),
		"yesterday"
	);

	let formatted = format.format(2.0, RelativeTimeUnit::Day).unwrap();
	let parts = format.format_to_parts(2.0, RelativeTimeUnit::Day).unwrap();
	let mut reconstructed = String::new();
	let mut saw_integer = false;
	for index in 0..parts.length() {
		let part = parts.get_unchecked(index);
		reconstructed.push_str(&String::from(part.value()));
		if part.type_() == "integer" {
			saw_integer = true;
			assert_eq!(part.unit(), Some(JsString::from("day")));
		}
	}
	assert_eq!(formatted, reconstructed);
	assert!(saw_integer);
}

#[test]
fn duration_format_dictionary_options_and_parts() {
	let duration = Duration::new();
	duration.set_years(1.0);
	duration.set_months(2.0);
	duration.set_weeks(3.0);
	duration.set_days(4.0);
	duration.set_hours(5.0);
	duration.set_minutes(6.0);
	duration.set_seconds(7.0);
	duration.set_milliseconds(8.0);
	duration.set_microseconds(9.0);
	duration.set_nanoseconds(10.0);
	assert_eq!(duration.years(), Some(1.0));
	assert_eq!(duration.months(), Some(2.0));
	assert_eq!(duration.weeks(), Some(3.0));
	assert_eq!(duration.days(), Some(4.0));
	assert_eq!(duration.hours(), Some(5.0));
	assert_eq!(duration.minutes(), Some(6.0));
	assert_eq!(duration.seconds(), Some(7.0));
	assert_eq!(duration.milliseconds(), Some(8.0));
	assert_eq!(duration.microseconds(), Some(9.0));
	assert_eq!(duration.nanoseconds(), Some(10.0));

	let options = DurationFormatOptions::new();
	options.set_locale_matcher(LocaleMatcher::Lookup);
	options.set_numbering_system("latn");
	options.set_style(DurationFormatStyle::Long);
	options.set_years(DurationUnitStyle::Long);
	options.set_years_display(DurationUnitDisplay::Always);
	options.set_months(DurationUnitStyle::Short);
	options.set_months_display(DurationUnitDisplay::Auto);
	options.set_weeks(DurationUnitStyle::Narrow);
	options.set_weeks_display(DurationUnitDisplay::Always);
	options.set_days(DurationUnitStyle::Long);
	options.set_days_display(DurationUnitDisplay::Auto);
	options.set_hours(DurationTimeUnitStyle::Short);
	options.set_hours_display(DurationUnitDisplay::Always);
	options.set_minutes(DurationTimeUnitStyle::Narrow);
	options.set_minutes_display(DurationUnitDisplay::Auto);
	options.set_seconds(DurationTimeUnitStyle::Long);
	options.set_seconds_display(DurationUnitDisplay::Always);
	options.set_milliseconds(DurationSubsecondUnitStyle::Short);
	options.set_milliseconds_display(DurationUnitDisplay::Auto);
	options.set_microseconds(DurationSubsecondUnitStyle::Narrow);
	options.set_microseconds_display(DurationUnitDisplay::Always);
	options.set_nanoseconds(DurationSubsecondUnitStyle::Long);
	options.set_nanoseconds_display(DurationUnitDisplay::Auto);
	options.set_fractional_digits(3);
	assert_eq!(options.locale_matcher(), Some(LocaleMatcher::Lookup));
	assert_eq!(options.numbering_system(), Some(JsString::from("latn")));
	assert_eq!(options.style(), Some(DurationFormatStyle::Long));
	assert_eq!(options.years(), Some(DurationUnitStyle::Long));
	assert_eq!(options.years_display(), Some(DurationUnitDisplay::Always));
	assert_eq!(options.months(), Some(DurationUnitStyle::Short));
	assert_eq!(options.months_display(), Some(DurationUnitDisplay::Auto));
	assert_eq!(options.weeks(), Some(DurationUnitStyle::Narrow));
	assert_eq!(options.weeks_display(), Some(DurationUnitDisplay::Always));
	assert_eq!(options.days(), Some(DurationUnitStyle::Long));
	assert_eq!(options.days_display(), Some(DurationUnitDisplay::Auto));
	assert_eq!(options.hours(), Some(DurationTimeUnitStyle::Short));
	assert_eq!(options.hours_display(), Some(DurationUnitDisplay::Always));
	assert_eq!(options.minutes(), Some(DurationTimeUnitStyle::Narrow));
	assert_eq!(options.minutes_display(), Some(DurationUnitDisplay::Auto));
	assert_eq!(options.seconds(), Some(DurationTimeUnitStyle::Long));
	assert_eq!(options.seconds_display(), Some(DurationUnitDisplay::Always));
	assert_eq!(
		options.milliseconds(),
		Some(DurationSubsecondUnitStyle::Short)
	);
	assert_eq!(
		options.milliseconds_display(),
		Some(DurationUnitDisplay::Auto)
	);
	assert_eq!(
		options.microseconds(),
		Some(DurationSubsecondUnitStyle::Narrow)
	);
	assert_eq!(
		options.microseconds_display(),
		Some(DurationUnitDisplay::Always)
	);
	assert_eq!(
		options.nanoseconds(),
		Some(DurationSubsecondUnitStyle::Long)
	);
	assert_eq!(
		options.nanoseconds_display(),
		Some(DurationUnitDisplay::Auto)
	);
	assert_eq!(options.fractional_digits(), Some(3));

	let format = DurationFormat::new_with_locales_and_options(&locale("en-US"), &options).unwrap();
	let resolved = format.resolved_options();
	assert!(resolved.locale().starts_with("en"));
	assert_eq!(resolved.numbering_system(), "latn");
	assert_eq!(resolved.style(), Some(DurationFormatStyle::Long));
	assert_eq!(resolved.years(), Some(DurationUnitStyle::Long));
	assert_eq!(resolved.years_display(), Some(DurationUnitDisplay::Always));
	assert_eq!(resolved.months(), Some(DurationUnitStyle::Short));
	assert_eq!(resolved.months_display(), Some(DurationUnitDisplay::Auto));
	assert_eq!(resolved.weeks(), Some(DurationUnitStyle::Narrow));
	assert_eq!(resolved.weeks_display(), Some(DurationUnitDisplay::Always));
	assert_eq!(resolved.days(), Some(DurationUnitStyle::Long));
	assert_eq!(resolved.days_display(), Some(DurationUnitDisplay::Auto));
	assert_eq!(resolved.hours(), Some(DurationTimeUnitStyle::Short));
	assert_eq!(resolved.hours_display(), Some(DurationUnitDisplay::Always));
	assert_eq!(resolved.minutes(), Some(DurationTimeUnitStyle::Narrow));
	assert_eq!(resolved.minutes_display(), Some(DurationUnitDisplay::Auto));
	assert_eq!(resolved.seconds(), Some(DurationTimeUnitStyle::Long));
	assert_eq!(
		resolved.seconds_display(),
		Some(DurationUnitDisplay::Always)
	);
	assert_eq!(
		resolved.milliseconds(),
		Some(DurationSubsecondUnitStyle::Short)
	);
	assert_eq!(
		resolved.milliseconds_display(),
		Some(DurationUnitDisplay::Auto)
	);
	assert_eq!(
		resolved.microseconds(),
		Some(DurationSubsecondUnitStyle::Narrow)
	);
	assert_eq!(
		resolved.microseconds_display(),
		Some(DurationUnitDisplay::Always)
	);
	assert_eq!(
		resolved.nanoseconds(),
		Some(DurationSubsecondUnitStyle::Long)
	);
	assert_eq!(
		resolved.nanoseconds_display(),
		Some(DurationUnitDisplay::Auto)
	);
	assert_eq!(resolved.fractional_digits(), Some(3));

	let formatted = format.format(&duration).unwrap();
	let parts = format.format_to_parts(&duration).unwrap();
	let mut reconstructed = String::new();
	let mut saw_integer = false;
	let mut saw_unit = false;
	let mut units = [false; 10];
	for index in 0..parts.length() {
		let part = parts.get_unchecked(index);
		reconstructed.push_str(&String::from(part.value()));
		match part.type_().unwrap() {
			DurationFormatPartType::Integer => saw_integer = true,
			DurationFormatPartType::Unit => saw_unit = true,
			_ => {}
		}
		if let Some(unit) = part.unit() {
			units[match unit {
				DurationUnit::Year => 0,
				DurationUnit::Month => 1,
				DurationUnit::Week => 2,
				DurationUnit::Day => 3,
				DurationUnit::Hour => 4,
				DurationUnit::Minute => 5,
				DurationUnit::Second => 6,
				DurationUnit::Millisecond => 7,
				DurationUnit::Microsecond => 8,
				DurationUnit::Nanosecond => 9,
			}] = true;
		}
	}
	assert_eq!(formatted, reconstructed);
	assert!(saw_integer && saw_unit);
	assert!(units.into_iter().all(core::convert::identity));

	let supported = DurationFormat::supported_locales_of(&locale("en-US")).unwrap();
	assert_ne!(supported.length(), 0);
	let matcher = LocaleMatcherOptions::new();
	matcher.set_locale_matcher(LocaleMatcher::Lookup);
	assert_ne!(
		DurationFormat::supported_locales_of_with_options(&locale("en-US"), &matcher)
			.unwrap()
			.length(),
		0
	);

	let empty = Duration::new();
	assert!(format.format(&empty).is_err());
	assert!(format.format_to_parts(&empty).is_err());
	let inconsistent = Duration::new();
	inconsistent.set_hours(1.0);
	inconsistent.set_minutes(-1.0);
	assert!(format.format(&inconsistent).is_err());
	assert!(format.format_to_parts(&inconsistent).is_err());
	let invalid_options = DurationFormatOptions::new();
	invalid_options.set_fractional_digits(10);
	assert!(
		DurationFormat::new_with_locales_and_options(&locale("en-US"), &invalid_options).is_err()
	);
	assert!(DurationFormat::new_with_locales(&locale("not_a_locale")).is_err());
}

#[test]
fn invalid_locales_and_options_are_errors() {
	let invalid_locale = locale("not_a_locale");
	assert!(Collator::new_with_locales(&invalid_locale).is_err());
	assert!(Segmenter::new_with_locales(&invalid_locale).is_err());

	let collator_options = CollatorOptions::new();
	collator_options.set_collation("invalid!");
	assert!(Collator::new_with_locales_and_options(&locale("en-US"), &collator_options).is_err());

	let object = Object::new();
	let value = JsValue::from(JsString::from("invalid"));
	assert!(Reflect::set_str(object.as_ref(), "granularity", &value).unwrap());
	let segmenter_options = SegmenterOptions::unchecked_from(object.into());
	assert!(Segmenter::new_with_locales_and_options(&locale("en-US"), &segmenter_options).is_err());
}

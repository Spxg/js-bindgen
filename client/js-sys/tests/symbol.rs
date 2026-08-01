use js_bindgen_test::test;
use js_sys::Symbol;

#[test]
fn symbols_are_unique() {
	let first = Symbol::new_with_description("local");
	let second = Symbol::new_with_description("local");

	assert_ne!(first, second);
	assert_eq!(first.description().unwrap(), "local");
	assert_eq!(first.to_js_string(), "Symbol(local)");
	assert_eq!(first.value_of(), first);
	assert!(Symbol::new().description().is_none());
}

#[test]
fn global_registry() {
	let first = Symbol::for_("shared");
	let second = Symbol::for_("shared");

	assert_eq!(first, second);
	assert_eq!(Symbol::key_for(&first).unwrap(), "shared");
	assert!(Symbol::key_for(&Symbol::new()).is_none());
}

#[test]
fn well_known_symbols_are_stable() {
	assert_eq!(Symbol::iterator(), Symbol::iterator());
	assert_eq!(Symbol::async_iterator(), Symbol::async_iterator());
	assert_ne!(Symbol::iterator(), Symbol::async_iterator());
}

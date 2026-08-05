use js_bindgen_test::test;
use js_sys::Symbol;

#[test]
fn constructor_static_and_instance_members() {
	let local = Symbol::new_with_description("local");
	assert_eq!(local.description().unwrap(), "local");

	let shared = Symbol::for_("shared");
	assert_eq!(Symbol::key_for(&shared).unwrap(), "shared");
	assert_eq!(Symbol::iterator(), Symbol::iterator());
}

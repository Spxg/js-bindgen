use js_bindgen_test::test;
use js_sys::hazard::JsCast;
use js_sys::{
	AsyncGenerator, AsyncIterable, AsyncIterator, Generator, Iterable, JsFuture, JsIterator,
	JsString, JsValue, js_sys,
};

js_bindgen::embed_js!(
	module = "generator",
	name = "sync.values",
	"() => (function* () {{",
	"    const received = yield 'first'",
	"    yield received",
	"    return 'done'",
	"}})()",
);
js_bindgen::embed_js!(
	module = "generator",
	name = "sync.catches",
	"() => (function* () {{",
	"    try {{ yield 'ready' }} catch (error) {{ yield error }}",
	"}})()",
);
js_bindgen::embed_js!(
	module = "generator",
	name = "sync.next_throws",
	"() => (function* () {{ throw new Error('next') }})()",
);
js_bindgen::embed_js!(
	module = "generator",
	name = "sync.return_throws",
	"() => (function* () {{",
	"    try {{ yield 'ready' }} finally {{ throw new Error('return') }}",
	"}})()",
);
js_bindgen::embed_js!(
	module = "generator",
	name = "async.values",
	"() => (async function* () {{",
	"    const received = yield 'first'",
	"    yield received",
	"    return 'done'",
	"}})()",
);
js_bindgen::embed_js!(
	module = "generator",
	name = "async.catches",
	"() => (async function* () {{",
	"    try {{ yield 'ready' }} catch (error) {{ yield error }}",
	"}})()",
);
js_bindgen::embed_js!(
	module = "generator",
	name = "async.next_rejects",
	"() => (async function* () {{ throw new Error('next') }})()",
);

#[js_sys]
extern "js-sys" {
	#[js_sys(js_embed = "sync.values")]
	fn sync_values() -> Generator<JsString, JsString, JsString>;

	#[js_sys(js_embed = "sync.catches")]
	fn sync_catches() -> Generator;

	#[js_sys(js_embed = "sync.next_throws")]
	fn sync_next_throws() -> Generator;

	#[js_sys(js_embed = "sync.return_throws")]
	fn sync_return_throws() -> Generator;

	#[js_sys(js_embed = "async.values")]
	fn async_values() -> AsyncGenerator<JsString, JsString, JsString>;

	#[js_sys(js_embed = "async.catches")]
	fn async_catches() -> AsyncGenerator;

	#[js_sys(js_embed = "async.next_rejects")]
	fn async_next_rejects() -> AsyncGenerator;
}

fn assert_sync_relations<T>()
where
	T: Iterable<Item = JsString> + AsRef<JsIterator<JsString>>,
{
}

fn assert_async_relations<T>()
where
	T: AsyncIterable<Item = JsString> + AsRef<AsyncIterator<JsString>>,
{
}

#[test]
fn generator_protocol_and_overloads() {
	assert_sync_relations::<Generator<JsString, JsString, JsString>>();

	let generator = sync_values();
	let first = generator.next().unwrap();
	assert!(!first.done().unwrap());
	assert_eq!(first.value().unwrap(), JsString::from("first").into());

	let sent = JsString::from("sent");
	let second = generator.next_with(&sent).unwrap();
	assert!(!second.done().unwrap());
	assert_eq!(second.value().unwrap(), sent.into());

	let done = generator.next().unwrap();
	assert!(done.done().unwrap());
	assert_eq!(done.value().unwrap(), JsString::from("done").into());

	let generator = sync_values();
	let returned = generator.return_with(&JsString::from("stopped")).unwrap();
	assert!(returned.done().unwrap());
	assert_eq!(returned.value().unwrap(), JsString::from("stopped").into());

	let generator: Generator = Generator::unchecked_from(sync_values().into());
	let returned = generator.return_().unwrap();
	assert!(returned.done().unwrap());
	assert_eq!(returned.value().unwrap(), JsValue::UNDEFINED);
}

#[test]
fn generator_throw_and_exceptions() {
	let generator = sync_catches();
	let ready = JsValue::from(JsString::from("ready"));
	let caught_value = JsValue::from(JsString::from("caught"));
	assert_eq!(generator.next().unwrap().value().unwrap(), ready);
	let caught = generator.throw_with(&caught_value).unwrap();
	assert!(!caught.done().unwrap());
	assert_eq!(caught.value().unwrap(), caught_value);

	let generator = sync_catches();
	generator.next().unwrap();
	let caught = generator.throw().unwrap();
	assert_eq!(caught.value().unwrap(), JsValue::UNDEFINED);

	assert!(sync_next_throws().next().is_err());

	let generator = sync_return_throws();
	generator.next().unwrap();
	assert!(generator.return_().is_err());
}

#[test]
async fn async_generator_protocol_and_overloads() {
	assert_async_relations::<AsyncGenerator<JsString, JsString, JsString>>();

	let generator = async_values();
	let first = JsFuture::from(generator.next()).await.unwrap();
	assert!(!first.done().unwrap());
	assert_eq!(first.value().unwrap(), JsString::from("first").into());

	let sent = JsString::from("sent");
	let second = JsFuture::from(generator.next_with(&sent)).await.unwrap();
	assert!(!second.done().unwrap());
	assert_eq!(second.value().unwrap(), sent.into());

	let done = JsFuture::from(generator.next()).await.unwrap();
	assert!(done.done().unwrap());
	assert_eq!(done.value().unwrap(), JsString::from("done").into());

	let generator = async_values();
	let returned = JsFuture::from(generator.return_with(&JsString::from("stopped")))
		.await
		.unwrap();
	assert!(returned.done().unwrap());
	assert_eq!(returned.value().unwrap(), JsString::from("stopped").into());

	let generator: AsyncGenerator = AsyncGenerator::unchecked_from(async_values().into());
	let returned = JsFuture::from(generator.return_()).await.unwrap();
	assert!(returned.done().unwrap());
	assert_eq!(returned.value().unwrap(), JsValue::UNDEFINED);
}

#[test]
async fn async_generator_throw_and_exceptions() {
	let generator = async_catches();
	JsFuture::from(generator.next()).await.unwrap();
	let caught_value = JsValue::from(JsString::from("caught"));
	let caught = JsFuture::from(generator.throw_with(&caught_value))
		.await
		.unwrap();
	assert!(!caught.done().unwrap());
	assert_eq!(caught.value().unwrap(), caught_value);

	let generator = async_catches();
	JsFuture::from(generator.next()).await.unwrap();
	let caught = JsFuture::from(generator.throw()).await.unwrap();
	assert_eq!(caught.value().unwrap(), JsValue::UNDEFINED);

	let rejected = JsFuture::from(async_next_rejects().next()).await;
	assert!(rejected.is_err());
}

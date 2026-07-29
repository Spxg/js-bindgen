use std::env;

use proc_macro2::TokenStream;
use quote::ToTokens;
#[cfg(any(feature = "file", test))]
use syn::File;
use syn::parse::Parser;
use syn::{Attribute, Error, ForeignItem, Item, ItemForeignMod, LitStr, Path, meta};

use crate::function::FunctionImport;
use crate::{Function, FunctionJsOutput, FunctionOperation, Hygiene, ImportManager, Type};

pub fn r#macro(
	attr: TokenStream,
	item: TokenStream,
	imports: Option<&mut ImportManager>,
) -> Result<TokenStream, TokenStream> {
	match syn::parse2(item).map_err(Error::into_compile_error)? {
		Item::ForeignMod(foreign_mod) => internal(attr, foreign_mod, None, imports)
			.map(GeneratedItems::into_token_stream)
			.map_err(|(output, error)| {
				let error = error.into_compile_error();

				if let Some(output) = output {
					let mut output = output.into_token_stream();
					output.extend(error);
					output
				} else {
					error
				}
			}),
		Item::Fn(function) => {
			crate::export::r#macro(attr, &function, None).map_err(Error::into_compile_error)
		}
		item => Err(
			Error::new_spanned(item, "expected an extern block or function").into_compile_error(),
		),
	}
}

pub(crate) fn internal(
	attr: TokenStream,
	mut foreign_mod: ItemForeignMod,
	crate_: Option<&str>,
	imports: Option<&mut ImportManager>,
) -> Result<GeneratedItems, (Option<GeneratedItems>, Error)> {
	let mut error = ErrorStack::new();

	let mut js_sys: Option<Path> = None;
	let mut namespace: Option<String> = None;

	if let Err(e) = meta::parser(|meta| {
		if meta.path.is_ident("js_sys") {
			if imports.is_some() {
				Err(meta.error("`js_sys` attribute only allowed with proc-macro hygiene"))
			} else if js_sys.is_some() {
				Err(meta.error("duplicate attribute"))
			} else {
				js_sys = Some(meta.value()?.parse()?);
				Ok(())
			}
		} else if meta.path.is_ident("namespace") {
			if namespace.is_some() {
				Err(meta.error("duplicate attribute"))
			} else {
				namespace = Some(meta.value()?.parse::<LitStr>()?.value());
				Ok(())
			}
		} else {
			Err(meta.error("unsupported attribute"))
		}
	})
	.parse2(attr)
	{
		error.push(e);
	}

	let mut hygiene = if let Some(imports) = imports {
		Hygiene::Imports(imports)
	} else {
		Hygiene::Hygiene {
			js_sys: js_sys.as_ref(),
		}
	};

	for attr in foreign_mod
		.attrs
		.extract_if(.., |attr| attr.path().is_ident("js_sys"))
	{
		error.push(Error::new_spanned(
			attr,
			"`js_sys` attribute not supported at that position",
		));
	}

	let mut output = GeneratedItems::default();
	let mut function_imports = Vec::new();

	if foreign_mod
		.abi
		.name
		.as_ref()
		.is_some_and(|value| value.value() != "js-sys")
	{
		error.push(Error::new_spanned(
			&foreign_mod.abi.name,
			"expected `js-sys` ABI",
		));
	}

	for item in foreign_mod.items {
		match item {
			ForeignItem::Fn(mut item) => {
				let mut js_output = FunctionJsOutput::default();

				for attr in item
					.attrs
					.extract_if(.., |attr| attr.path().is_ident("js_sys"))
				{
					if let Err(e) = attr.parse_nested_meta(|meta| {
						let FunctionJsOutput::Generate {
							js_name,
							static_of,
							operation,
						} = &mut js_output
						else {
							return Err(meta.error("found duplicate/incompatible attribute"));
						};

						if meta.path.is_ident("js_name") {
							*js_name = Some(meta.value()?.parse::<LitStr>()?.value());
							Ok(())
						} else if meta.path.is_ident("js_import") {
							if meta.input.is_empty() {
								if js_name.is_some() || static_of.is_some() || operation.is_some() {
									return Err(
										meta.error("found duplicate/incompatible attribute")
									);
								}
								js_output = FunctionJsOutput::Import;
								Ok(())
							} else {
								Err(meta.error("`js_import` supports no values"))
							}
						} else if meta.path.is_ident("js_embed") {
							if js_name.is_some() || static_of.is_some() || operation.is_some() {
								return Err(meta.error("found duplicate/incompatible attribute"));
							}
							js_output =
								FunctionJsOutput::Embed(meta.value()?.parse::<LitStr>()?.value());
							Ok(())
						} else if meta.path.is_ident("static_of") {
							if static_of.replace(meta.value()?.parse()?).is_some() {
								Err(meta.error("duplicate attribute"))
							} else {
								Ok(())
							}
						} else if meta.path.is_ident("constructor") {
							set_operation(&meta, operation, FunctionOperation::Constructor)
						} else if meta.path.is_ident("getter") {
							set_operation(&meta, operation, FunctionOperation::Getter)
						} else if meta.path.is_ident("setter") {
							set_operation(&meta, operation, FunctionOperation::Setter)
						} else {
							Err(meta.error("unsupported attribute"))
						}
					}) {
						error.push(e);
					}
				}

				let crate_ = if let Some(crate_) = crate_ {
					crate_
				} else {
					&env::var("CARGO_CRATE_NAME").expect("`CARGO_CRATE_NAME` not found")
				};

				match Function::new(&mut hygiene, js_output, namespace.as_deref(), crate_, item) {
					Ok(function) => {
						let (function, import) = function.into_parts();
						output.push(&function);
						function_imports.push(import);
					}
					Err(e) => error.push(e),
				}
			}
			ForeignItem::Type(mut item) => {
				if let Some(attr) = item
					.attrs
					.extract_if(.., |attr| attr.path().is_ident("js_sys"))
					.next()
				{
					error.push(Error::new_spanned(attr, "unsupported attribute"));
				}

				for item in Type::new(&mut hygiene, item) {
					output.push(&item);
				}
			}
			item => {
				error.push(Error::new_spanned(
					item,
					"expected foreign function or type ",
				));
			}
		}
	}

	output.extend(render_import_groups(function_imports));

	if let Some(error) = error.resolve() {
		Err((Some(output), error))
	} else {
		Ok(output)
	}
}

#[derive(Debug, Default)]
pub(crate) struct GeneratedItems(TokenStream);

impl GeneratedItems {
	fn push(&mut self, item: &impl ToTokens) {
		item.to_tokens(&mut self.0);
	}

	fn extend(&mut self, items: TokenStream) {
		self.0.extend(items);
	}

	pub(crate) fn into_token_stream(self) -> TokenStream {
		self.0
	}

	#[cfg(any(feature = "file", test))]
	pub(crate) fn into_items(self) -> Result<Vec<Item>, Error> {
		Ok(syn::parse2::<File>(self.0)?.items)
	}
}

struct ImportGroup {
	attrs: Vec<Attribute>,
	descriptors: Vec<TokenStream>,
	has_js: bool,
	r#macro: Path,
}

fn render_import_groups(imports: Vec<FunctionImport>) -> TokenStream {
	let mut groups: Vec<ImportGroup> = Vec::new();

	for import in imports {
		// One section static may cover every import with the same conditional
		// compilation boundary. Keeping the original attributes on a containing
		// item also preserves arbitrary `cfg_attr` expansions.
		if let Some(group) = groups.iter_mut().find(|group| group.attrs == import.attrs) {
			group.descriptors.push(import.descriptor);
			group.has_js |= import.has_js;
		} else {
			groups.push(ImportGroup {
				attrs: import.attrs,
				descriptors: vec![import.descriptor],
				has_js: import.has_js,
				r#macro: import.r#macro,
			});
		}
	}

	groups
		.into_iter()
		.fold(TokenStream::new(), |mut output, group| {
			let ImportGroup {
				attrs,
				descriptors,
				has_js,
				r#macro,
			} = group;
			let js = has_js.then(|| {
				quote::quote! {
					const JS_CAPACITY: ::core::primitive::usize =
						#r#macro::import_js_batch_capacity(IMPORTS);

					#[used]
					#[unsafe(link_section = "js_bindgen.import")]
					static JS_SECTION: #r#macro::ImportBatchSection<JS_CAPACITY> =
						#r#macro::import_js_batch::<JS_CAPACITY>(IMPORTS);
				}
			});

			output.extend(quote::quote! {
				const _: () = {
					#(#attrs)*
					fn import_sections() {
						const IMPORTS: &[#r#macro::ImportDescriptor] = &[#(#descriptors),*];
						const WAT_CAPACITY: ::core::primitive::usize =
							#r#macro::import_wat_batch_capacity(IMPORTS);

						#[used]
						#[unsafe(link_section = "js_bindgen.wat")]
						static WAT_SECTION: #r#macro::ImportBatchSection<WAT_CAPACITY> =
							#r#macro::import_wat_batch::<WAT_CAPACITY>(IMPORTS);

						#js
					}
				};
			});
			output
		})
}

fn set_operation(
	meta: &meta::ParseNestedMeta<'_>,
	operation: &mut Option<FunctionOperation>,
	value: FunctionOperation,
) -> Result<(), Error> {
	if operation.replace(value).is_some() {
		return Err(meta.error("found duplicate/incompatible attribute"));
	}

	Ok(())
}

pub(crate) struct ErrorStack(Option<Error>);

impl ErrorStack {
	pub(crate) fn new() -> Self {
		Self(None)
	}

	pub(crate) fn push(&mut self, error: Error) {
		match &mut self.0 {
			Some(this) => this.combine(error),
			None => self.0 = Some(error),
		}
	}

	pub(crate) fn resolve(self) -> Option<Error> {
		self.0
	}
}

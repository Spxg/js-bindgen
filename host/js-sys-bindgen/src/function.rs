use std::mem;
use std::ops::DerefMut;
use std::string::ToString;

use itertools::Itertools;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
	Attribute, Error, FnArg, ForeignItemFn, GenericArgument, GenericParam, Generics, Ident, Item,
	ItemFn, ItemImpl, Pat, PatIdent, PatType, Path, PathArguments, Receiver, Result, ReturnType,
	Signature, Stmt, Token, Type, TypePath, TypeReference, parse_quote, parse_quote_spanned,
};

use crate::Hygiene;

pub enum Function {
	Fn(ItemFn),
	Impl(ItemImpl),
}

pub enum FunctionJsOutput {
	Generate {
		js_name: Option<String>,
		static_of: Option<Path>,
		operation: Option<FunctionOperation>,
	},
	Embed(String),
	Import,
}

#[derive(Clone, Copy)]
pub enum FunctionOperation {
	Constructor,
	Getter,
	Setter,
}

struct State<'a> {
	crate_: &'a str,
	namespace: Option<&'a str>,
	js_bindgen: Path,
	r#macro: Path,
	import_name: String,
	foreign_name: String,
	inputs: Vec<InputArg>,
	output_ty: Option<Type>,
	impl_generic_params: TokenStream,
	binding: JsBinding,
	span: Span,
}

struct InputArg {
	abi_type: Type,
	rust_name: Ident,
	wat_name: syn::LitStr,
	slot_names: [Ident; 4],
	type_override: bool,
}

enum JsBinding {
	Generate(GeneratedBinding),
	Embed(String),
	Import,
}

struct GeneratedBinding {
	target: JsTarget,
	operation: JsOperation,
	js_name: String,
}

enum JsTarget {
	Global,
	Instance(Path),
	Static(Path),
}

#[derive(Clone, Copy)]
enum JsOperation {
	Call,
	Construct,
	Getter,
	Setter,
}

impl InputArg {
	fn new(index: usize, abi_type: Type, rust_name: Ident, type_override: bool) -> Self {
		let span = Span::mixed_site();
		let base = format!("arg{index}");
		let slot_name = |slot| Ident::new(&format!("{base}_{slot}"), span);

		Self {
			abi_type,
			rust_name,
			wat_name: syn::LitStr::new(&base, span),
			slot_names: [slot_name(0), slot_name(1), slot_name(2), slot_name(3)],
			type_override,
		}
	}
}

impl Function {
	pub fn new(
		hygiene: &mut Hygiene<'_>,
		js_output: FunctionJsOutput,
		namespace: Option<&str>,
		crate_: &str,
		item: ForeignItemFn,
	) -> Result<Self> {
		if let Some(constness) = item.sig.constness {
			return Err(Error::new_spanned(
				constness,
				"`const` functions are not supported",
			));
		}

		if let Some(asyncness) = item.sig.asyncness {
			return Err(Error::new_spanned(
				asyncness,
				"`async` functions are not supported",
			));
		}

		if let Some(variadic) = &item.sig.variadic {
			return Err(Error::new_spanned(
				variadic,
				"variadic functions are not supported",
			));
		}

		let span = item.span();
		let ForeignItemFn {
			attrs,
			vis,
			mut sig,
			..
		} = item;
		let outer_attrs: Vec<_> = attrs
			.iter()
			.filter(|attr| {
				let path = attr.path();
				path.is_ident("cfg") || path.is_ident("cfg_attr")
			})
			.cloned()
			.collect();

		let state = State::parse(
			crate_,
			js_output,
			namespace,
			hygiene,
			&outer_attrs,
			&mut sig,
			span,
		)?;
		let wat = state.wat();
		let js = state.js();
		let State {
			r#macro,
			foreign_name,
			inputs,
			output_ty,
			impl_generic_params,
			binding,
			..
		} = state;
		let ident = &sig.ident;
		let split_inputs = inputs.iter().map(|input| {
			let InputArg {
				abi_type,
				rust_name,
				slot_names: [slot1, slot2, slot3, slot4],
				type_override,
				..
			} = input;
			let split_input = if *type_override {
				quote_spanned!(span=> unsafe {
					#r#macro::split_input_as::<#abi_type>(#rust_name)
				})
			} else {
				quote_spanned!(span=> #r#macro::split_input::<#abi_type>(#rust_name))
			};

			quote_spanned! {span=>
				let (#slot1, #slot2, #slot3, #slot4) = #split_input;
			}
		});
		let foreign_input_names: Vec<_> = inputs.iter().flat_map(|arg| &arg.slot_names).collect();
		let foreign_input_tys: Vec<_> = inputs
			.iter()
			.flat_map(|arg| {
				let ty = &arg.abi_type;

				[
					quote_spanned!(span=> #r#macro::InputSlot1<#ty>),
					quote_spanned!(span=> #r#macro::InputSlot2<#ty>),
					quote_spanned!(span=> #r#macro::InputSlot3<#ty>),
					quote_spanned!(span=> #r#macro::InputSlot4<#ty>),
				]
			})
			.collect();
		let foreign_output = output_ty.as_ref().map_or_else(
			TokenStream::new,
			|ty| quote_spanned!(span=> -> #r#macro::OutputRet<#ty>),
		);

		let foreign_call = quote_spanned! {span=> {
			#(#split_inputs)*
			unsafe { #ident(#(#foreign_input_names),*) }
		}};
		let foreign_call = if output_ty.is_some() {
			quote_spanned!(span=> #r#macro::join_output(#foreign_call))
		} else {
			quote_spanned!(span=> #foreign_call;)
		};

		let item_fn = parse_quote_spanned! {span=>
			#(#attrs)*
			#vis #sig {
				#wat

				#js

				unsafe extern "C" {
					#[link_name = #foreign_name]
					fn #ident(#(#foreign_input_names: #foreign_input_tys),*) #foreign_output;
				}

				#foreign_call
			}
		};

		if let Some(owner) = binding.owner() {
			Ok(Self::Impl(parse_quote_spanned! {span=>
				impl #impl_generic_params #owner {
					#item_fn
				}
			}))
		} else {
			Ok(Self::Fn(item_fn))
		}
	}
}

impl From<Function> for Item {
	fn from(value: Function) -> Self {
		match value {
			Function::Fn(item) => item.into(),
			Function::Impl(item) => item.into(),
		}
	}
}

impl ToTokens for Function {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		match self {
			Self::Fn(item) => item.to_tokens(tokens),
			Self::Impl(item) => item.to_tokens(tokens),
		}
	}
}

impl Default for FunctionJsOutput {
	fn default() -> Self {
		Self::Generate {
			js_name: None,
			static_of: None,
			operation: None,
		}
	}
}

impl<'a> State<'a> {
	fn parse(
		crate_: &'a str,
		js_output: FunctionJsOutput,
		namespace: Option<&'a str>,
		hygiene: &mut Hygiene<'_>,
		outer_attrs: &[Attribute],
		sig: &mut Signature,
		span: Span,
	) -> Result<Self> {
		let mut self_ty = None;

		let inputs = sig
			.inputs
			.iter_mut()
			.enumerate()
			.map(|(index, arg)| {
				if let FnArg::Typed(PatType { attrs, pat, ty, .. }) = arg
					&& let Pat::Ident(PatIdent {
						attrs: inner_attrs,
						by_ref: None,
						mutability: None,
						ident,
						subpat: None,
					}) = pat.deref_mut()
					&& inner_attrs.is_empty()
				{
					let mut r#type = None;

					if let Some(attr) = attrs
						.extract_if(.., |attr| attr.path().is_ident("js_sys"))
						.next()
					{
						attr.parse_nested_meta(|meta| {
							if meta.path.is_ident("type") {
								meta.input.parse::<Token![=]>()?;

								if r#type.replace(meta.input.parse::<Type>()?).is_some() {
									Err(meta.error("duplicate attribute"))
								} else {
									Ok(())
								}
							} else {
								Err(meta.error("unsupported attribute"))
							}
						})?;
					}

					let type_override = r#type.is_some();
					let r#type = r#type.unwrap_or_else(|| *ty.clone());

					Ok(InputArg::new(index, r#type, ident.clone(), type_override))
				} else if let FnArg::Receiver(Receiver {
					attrs,
					reference: None,
					mutability: None,
					self_token,
					colon_token: Some(_),
					ty,
				}) = arg && attrs.is_empty()
					&& let Type::Reference(TypeReference {
						and_token,
						lifetime: None,
						mutability: None,
						elem,
					}) = ty.deref_mut()
					&& let Type::Path(TypePath { qself: None, path }) = elem.deref_mut()
				{
					if !matches!(&js_output, FunctionJsOutput::Generate { .. }) {
						return Err(Error::new_spanned(
							path,
							"`self` is not supported with `js_import` and `js_embed`",
						));
					}

					self_ty = Some(path.clone());
					let js_value = hygiene.js_value(outer_attrs, span);
					Ok(InputArg::new(
						index,
						parse_quote! { #and_token #js_value },
						(*self_token).into(),
						true,
					))
				} else {
					Err(Error::new_spanned(arg, "unsupported arguments found"))
				}
			})
			.collect::<Result<Vec<_>>>()?;

		let binding = match js_output {
			FunctionJsOutput::Generate {
				js_name,
				static_of,
				operation,
			} => {
				if self_ty.is_some() && static_of.is_some() {
					return Err(Error::new(
						span,
						"`static_of` cannot be used with a `self` parameter",
					));
				}

				let operation = match operation {
					Some(FunctionOperation::Constructor) => JsOperation::Construct,
					Some(FunctionOperation::Getter) => JsOperation::Getter,
					Some(FunctionOperation::Setter) => JsOperation::Setter,
					None => JsOperation::Call,
				};
				let target = if matches!(operation, JsOperation::Construct) {
					if self_ty.is_some() {
						return Err(Error::new(
							span,
							"`constructor` cannot be used with a `self` parameter",
						));
					}
					if static_of.is_some() {
						return Err(Error::new(
							span,
							"`constructor` cannot be combined with `static_of`",
						));
					}

					JsTarget::Static(Self::constructor_owner(&sig.output)?)
				} else if let Some(static_of) = static_of {
					JsTarget::Static(static_of)
				} else if let Some(self_ty) = self_ty {
					JsTarget::Instance(self_ty)
				} else {
					JsTarget::Global
				};
				let argument_count =
					sig.inputs.len() - usize::from(matches!(&target, JsTarget::Instance(_)));

				match operation {
					JsOperation::Getter
						if argument_count != 0 || !matches!(&sig.output, ReturnType::Type(..)) =>
					{
						return Err(Error::new(
							span,
							"`getter` requires no arguments and a return value",
						));
					}
					JsOperation::Setter
						if argument_count != 1 || !matches!(&sig.output, ReturnType::Default) =>
					{
						return Err(Error::new(
							span,
							"`setter` requires one argument and no return value",
						));
					}
					_ => {}
				}

				let js_name = js_name.unwrap_or_else(|| {
					if matches!(operation, JsOperation::Construct) {
						Self::target_name(&target)
					} else {
						sig.ident.to_string()
					}
				});

				JsBinding::Generate(GeneratedBinding {
					target,
					operation,
					js_name,
				})
			}
			FunctionJsOutput::Embed(embed) => JsBinding::Embed(embed),
			FunctionJsOutput::Import => JsBinding::Import,
		};
		let import_name = binding.import_name(namespace, &sig.ident);
		let foreign_name = format!("{crate_}.{import_name}");

		let output_ty = match &sig.output {
			ReturnType::Default => None,
			ReturnType::Type(_, ty) => Some(*ty.clone()),
		};

		let impl_generic_params = Self::impl_generic_params(&binding, &mut sig.generics);

		let js_bindgen = hygiene.js_bindgen(outer_attrs, span);
		let r#macro = hygiene.r#macro(outer_attrs, span);

		Ok(Self {
			crate_,
			namespace,
			js_bindgen,
			r#macro,
			import_name,
			foreign_name,
			inputs,
			output_ty,
			impl_generic_params,
			binding,
			span,
		})
	}

	fn constructor_owner(output: &ReturnType) -> Result<Path> {
		let ReturnType::Type(_, output) = output else {
			return Err(Error::new_spanned(
				output,
				"`constructor` requires a return type",
			));
		};

		Self::constructor_owner_from_type(output)
	}

	fn constructor_owner_from_type(output: &Type) -> Result<Path> {
		let Type::Path(TypePath { qself: None, path }) = output else {
			return Err(Error::new_spanned(
				output,
				"`constructor` requires a path return type",
			));
		};
		let segment = path
			.segments
			.last()
			.expect("a type path always contains a segment");

		if segment.ident == "Result"
			&& let PathArguments::AngleBracketed(arguments) = &segment.arguments
			&& let Some(GenericArgument::Type(output)) = arguments.args.first()
		{
			return Self::constructor_owner_from_type(output);
		}

		Ok(path.clone())
	}

	fn target_name(target: &JsTarget) -> String {
		let JsTarget::Static(owner) = target else {
			unreachable!("only static targets have a type name");
		};

		owner
			.segments
			.last()
			.expect("a type path always contains a segment")
			.ident
			.to_string()
	}

	// Extract type generics from signature that are part of `impl <type>`.
	fn impl_generic_params(binding: &JsBinding, generics: &mut Generics) -> TokenStream {
		if let Some(owner) = binding.owner() {
			let mut fn_generic_params: Vec<_> =
				mem::take(&mut generics.params).into_iter().collect();

			let impl_generic_params: Vec<_> = fn_generic_params
				.extract_if(.., |param| {
					for path in &owner.segments {
						if let PathArguments::AngleBracketed(args) = &path.arguments {
							for arg in &args.args {
								match (&*param, arg) {
									(
										GenericParam::Lifetime(param),
										GenericArgument::Lifetime(arg),
									) if &param.lifetime == arg => {
										return true;
									}
									(
										GenericParam::Type(param),
										GenericArgument::Type(Type::Path(TypePath {
											qself: None,
											path,
										})),
									) => {
										if let Some(arg) = path.get_ident()
											&& &param.ident == arg
										{
											return true;
										}
									}
									_ => (),
								}
							}
						}
					}

					false
				})
				.collect();

			generics.params = fn_generic_params.into_iter().collect();

			if impl_generic_params.is_empty() {
				TokenStream::new()
			} else {
				let lt = generics.lt_token.unwrap();
				let gt = generics.gt_token.unwrap();

				quote!(#lt #(#impl_generic_params),* #gt)
			}
		} else {
			TokenStream::new()
		}
	}

	fn wat(&self) -> Stmt {
		let Self {
			crate_,
			js_bindgen,
			r#macro,
			import_name,
			foreign_name,
			inputs,
			output_ty,
			span,
			..
		} = self;
		let inputs = inputs.iter().map(|input| {
			let name = &input.wat_name;
			let ty = &input.abi_type;

			quote_spanned!(*span=> (#name, #ty))
		});
		let output = output_ty.iter();

		parse_quote_spanned! {*span=>
			#js_bindgen::unsafe_global_wat! {
				"{}",
				interpolate #r#macro::wat_import!(
					module = #crate_,
					import = #import_name,
					shim = #foreign_name,
					inputs = [#(#inputs),*],
					#(output = #output,)*
				),
			}
		}
	}

	fn js(&self) -> Option<Stmt> {
		let Self {
			crate_,
			js_bindgen,
			r#macro,
			import_name,
			inputs,
			output_ty,
			binding,
			span,
			..
		} = self;
		let input_tys: Vec<_> = inputs.iter().map(|input| &input.abi_type).collect();
		let input_names: Vec<_> = inputs.iter().map(|input| &input.wat_name).collect();
		let input_value_names: Vec<_> = inputs
			.iter()
			.map(|input| input.slot_names[0].to_string())
			.collect();
		let output_tys: Vec<_> = output_ty.iter().collect();

		let mut unique_inputs = Vec::new();

		for &ty in &input_tys {
			if !unique_inputs.contains(&ty) {
				unique_inputs.push(ty);
			}
		}

		let mut required_embeds = Vec::new();

		if let JsBinding::Embed(name) = binding {
			required_embeds.push(quote_spanned!(*span=> (#crate_, #name)));
		}

		for ty in &unique_inputs {
			required_embeds.push(quote_spanned!(*span=> #r#macro::js_input_embed::<#ty>()));
		}

		for &ty in &output_tys {
			required_embeds.push(quote_spanned!(*span=> #r#macro::js_output_embed::<#ty>()));
			required_embeds.push(quote_spanned!(*span=> #r#macro::js_result_embed::<#ty>()));
		}

		let required_embeds = if required_embeds.is_empty() {
			[].as_slice()
		} else {
			&[quote_spanned!(*span=> required_embeds = [#(#required_embeds),*])]
		};

		let input_names_joined = input_value_names.iter().join(", ");
		let js_inputs: Vec<_> = input_names
			.iter()
			.zip(input_tys.iter())
			.map(|(name, ty)| quote_spanned!(*span=> (#name, #ty)))
			.collect();
		let (direct_wrapper, direct_js_call, indirect_js_call) = match binding {
			JsBinding::Generate(binding) => {
				let call_inputs = if binding.has_receiver() {
					input_value_names.iter().skip(1).join(", ")
				} else {
					input_names_joined.clone()
				};
				let expression =
					binding.expression(self.namespace, &input_value_names, &call_inputs);

				if binding.requires_wrapper(self.namespace) {
					(true, expression.clone(), expression)
				} else {
					let path = binding.path(self.namespace, &input_value_names);
					(false, path.clone(), format!("{path}({input_names_joined})"))
				}
			}
			JsBinding::Embed(name) => {
				let path = format!("this.#jsEmbed.{crate_}['{name}']");
				(false, path.clone(), format!("{path}({input_names_joined})"))
			}
			JsBinding::Import => return None,
		};
		let output = output_ty.iter();

		Some(parse_quote_spanned! {*span=>
			#js_bindgen::import_js! {
				module = #crate_,
				name = #import_name,
				#(#required_embeds,)*
				"{}",
				interpolate #r#macro::js_import!(
					direct_wrapper = #direct_wrapper,
					direct_call = #direct_js_call,
					indirect_call = #indirect_js_call,
					inputs = [#(#js_inputs),*],
					#(output = #output,)*
				),
			}
		})
	}
}

impl JsBinding {
	fn owner(&self) -> Option<&Path> {
		let Self::Generate(binding) = self else {
			return None;
		};

		binding.owner()
	}

	fn import_name(&self, namespace: Option<&str>, rust_name: &Ident) -> String {
		let name = if let Self::Generate(binding) = self
			&& matches!(&binding.target, JsTarget::Static(_))
		{
			format!("{}.{}", binding.owner_name(), rust_name)
		} else {
			rust_name.to_string()
		};

		if let Some(namespace) = namespace {
			format!("{namespace}.{name}")
		} else {
			name
		}
	}
}

impl GeneratedBinding {
	fn owner(&self) -> Option<&Path> {
		match &self.target {
			JsTarget::Global => None,
			JsTarget::Instance(owner) | JsTarget::Static(owner) => Some(owner),
		}
	}

	fn owner_name(&self) -> &Ident {
		self.owner()
			.and_then(|owner| owner.segments.last())
			.map(|segment| &segment.ident)
			.expect("static and instance bindings always have an owner")
	}

	fn has_receiver(&self) -> bool {
		matches!(&self.target, JsTarget::Instance(_))
	}

	fn requires_wrapper(&self, namespace: Option<&str>) -> bool {
		namespace.is_some()
			|| !matches!(&self.target, JsTarget::Global)
			|| !matches!(self.operation, JsOperation::Call)
	}

	fn path(&self, namespace: Option<&str>, inputs: &[String]) -> String {
		match &self.target {
			JsTarget::Global => Self::global_path(namespace, &self.js_name),
			JsTarget::Instance(_) => format!("{}.{}", inputs[0], self.js_name),
			JsTarget::Static(_) if matches!(self.operation, JsOperation::Construct) => {
				Self::global_path(namespace, &self.js_name)
			}
			JsTarget::Static(_) => Self::global_path(
				namespace,
				&format!("{}.{}", self.owner_name(), self.js_name),
			),
		}
	}

	fn global_path(namespace: Option<&str>, name: &str) -> String {
		if let Some(namespace) = namespace {
			format!("globalThis.{namespace}.{name}")
		} else {
			format!("globalThis.{name}")
		}
	}

	fn expression(&self, namespace: Option<&str>, inputs: &[String], call_inputs: &str) -> String {
		let path = self.path(namespace, inputs);

		match self.operation {
			JsOperation::Call => format!("{path}({call_inputs})"),
			JsOperation::Construct => format!("new {path}({call_inputs})"),
			JsOperation::Getter => path,
			JsOperation::Setter => format!("{path} = {call_inputs}"),
		}
	}
}

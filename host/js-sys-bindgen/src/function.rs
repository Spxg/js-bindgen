use std::mem;
use std::ops::DerefMut;
use std::string::ToString;

#[cfg(feature = "macro")]
use itertools::Itertools;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
	Attribute, Error, FnArg, ForeignItemFn, GenericArgument, GenericParam, Generics, Ident, Item,
	Pat, PatIdent, PatType, Path, PathArguments, Receiver, Result, ReturnType, Signature, Token,
	Type, TypePath, TypeReference, parse_quote,
};

use crate::Hygiene;

pub struct Function {
	item: TokenStream,
	#[cfg(feature = "macro")]
	import: FunctionImport,
}

#[cfg(feature = "macro")]
pub(crate) struct FunctionImport {
	pub(crate) attrs: Vec<Attribute>,
	pub(crate) descriptor: TokenStream,
	pub(crate) has_js: bool,
	pub(crate) r#macro: Path,
}

pub enum FunctionJsOutput {
	Generate {
		js_name: Option<String>,
		static_of: Option<Path>,
		operation: Option<FunctionOperation>,
		variadic: bool,
	},
	Embed(String),
	Import,
}

pub enum FunctionOperation {
	Constructor,
	Getter(Option<String>),
	Setter(Option<String>),
}

struct State {
	r#macro: Path,
	foreign_name: String,
	inputs: Vec<InputArg>,
	output_ty: Option<Type>,
	impl_generic_params: TokenStream,
	binding: JsBinding,
}

struct InputArg {
	abi_type: Type,
	rust_name: Ident,
	#[cfg(feature = "macro")]
	wat_name: syn::LitStr,
	slot_names: [Ident; 4],
	type_override: bool,
}

enum JsBinding {
	Generate(GeneratedBinding),
	#[cfg(feature = "macro")]
	Embed(String),
	#[cfg(not(feature = "macro"))]
	Embed,
	Import,
}

struct GeneratedBinding {
	target: JsTarget,
	#[cfg(feature = "macro")]
	variadic: bool,
	#[cfg(feature = "macro")]
	operation: JsOperation,
	#[cfg(feature = "macro")]
	js_name: String,
	#[cfg(feature = "macro")]
	property_name: Option<String>,
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
			#[cfg(feature = "macro")]
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
		#[cfg(feature = "macro")]
		let import = state.import_descriptor(crate_, namespace, &sig.ident, &outer_attrs, span);
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

		let item_fn = quote_spanned! {span=>
			#(#attrs)*
			#vis #sig {
				unsafe extern "C" {
					#[link_name = #foreign_name]
					fn #ident(#(#foreign_input_names: #foreign_input_tys),*) #foreign_output;
				}

				#foreign_call
			}
		};

		let item = if let Some(owner) = binding.owner() {
			quote_spanned! {span=>
				impl #impl_generic_params #owner {
					#item_fn
				}
			}
		} else {
			item_fn
		};

		Ok(Self {
			item,
			#[cfg(feature = "macro")]
			import,
		})
	}

	#[cfg(feature = "macro")]
	pub(crate) fn into_parts(self) -> (TokenStream, FunctionImport) {
		(self.item, self.import)
	}
}

impl From<Function> for Item {
	fn from(value: Function) -> Self {
		syn::parse2(value.item).expect("generated function must be a valid Rust item")
	}
}

impl ToTokens for Function {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		self.item.to_tokens(tokens);
	}
}

impl Default for FunctionJsOutput {
	fn default() -> Self {
		Self::Generate {
			js_name: None,
			static_of: None,
			operation: None,
			variadic: false,
		}
	}
}

impl State {
	fn parse(
		crate_: &str,
		js_output: FunctionJsOutput,
		namespace: Option<&str>,
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
				variadic,
			} => {
				if self_ty.is_some() && static_of.is_some() {
					return Err(Error::new(
						span,
						"`static_of` cannot be used with a `self` parameter",
					));
				}

				let (operation, property_name) = match operation {
					Some(FunctionOperation::Constructor) => (JsOperation::Construct, None),
					Some(FunctionOperation::Getter(name)) => (
						JsOperation::Getter,
						Some(name.unwrap_or_else(|| sig.ident.to_string())),
					),
					Some(FunctionOperation::Setter(name)) => (
						JsOperation::Setter,
						Some(match name {
							Some(name) => name,
							None => Self::infer_setter_property(&sig.ident)?,
						}),
					),
					None => (JsOperation::Call, None),
				};
				if property_name.is_some() && js_name.is_some() {
					return Err(Error::new(
						span,
						"`js_name` cannot be combined with `getter` or `setter`; specify the field \
						 on the property operation",
					));
				}
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

				if variadic && argument_count == 0 {
					return Err(Error::new(
						span,
						"`variadic` requires at least one argument",
					));
				}
				if variadic && !matches!(operation, JsOperation::Call | JsOperation::Construct) {
					return Err(Error::new(
						span,
						"`variadic` cannot be combined with a property operation",
					));
				}

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

				#[cfg(feature = "macro")]
				let js_name = js_name.unwrap_or_else(|| {
					if matches!(operation, JsOperation::Construct) {
						Self::target_name(&target)
					} else {
						sig.ident.to_string()
					}
				});
				#[cfg(not(feature = "macro"))]
				let _ = (js_name, property_name);

				JsBinding::Generate(GeneratedBinding {
					target,
					#[cfg(feature = "macro")]
					variadic,
					#[cfg(feature = "macro")]
					operation,
					#[cfg(feature = "macro")]
					js_name,
					#[cfg(feature = "macro")]
					property_name,
				})
			}
			#[cfg(feature = "macro")]
			FunctionJsOutput::Embed(embed) => JsBinding::Embed(embed),
			#[cfg(not(feature = "macro"))]
			FunctionJsOutput::Embed(_) => JsBinding::Embed,
			FunctionJsOutput::Import => JsBinding::Import,
		};
		let import_name = binding.import_name(namespace, &sig.ident);
		let foreign_name = format!("{crate_}.{import_name}");

		let output_ty = match &sig.output {
			ReturnType::Default => None,
			ReturnType::Type(_, ty) => Some(*ty.clone()),
		};

		let impl_generic_params = Self::impl_generic_params(&binding, &mut sig.generics);

		let r#macro = hygiene.r#macro(outer_attrs, span);

		Ok(Self {
			r#macro,
			foreign_name,
			inputs,
			output_ty,
			impl_generic_params,
			binding,
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

	fn infer_setter_property(ident: &Ident) -> Result<String> {
		let name = ident.to_string();
		let Some(property) = name
			.strip_prefix("set_")
			.filter(|property| !property.is_empty())
		else {
			return Err(Error::new_spanned(
				ident,
				"`setter` cannot infer a field name; use `setter = \"field\"`",
			));
		};

		Ok(property.to_owned())
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

	#[cfg(feature = "macro")]
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

	#[cfg(feature = "macro")]
	fn import_descriptor(
		&self,
		crate_: &str,
		namespace: Option<&str>,
		rust_name: &Ident,
		attrs: &[Attribute],
		span: Span,
	) -> FunctionImport {
		let Self {
			r#macro,
			foreign_name,
			inputs,
			output_ty,
			binding,
			..
		} = self;
		let import_name = binding.import_name(namespace, rust_name);
		let input_descriptors = inputs.iter().map(|input| {
			let name = &input.wat_name;
			let ty = &input.abi_type;

			quote_spanned!(span=> #r#macro::import_input::<#ty>(#name))
		});
		let input_tys: Vec<_> = inputs.iter().map(|input| &input.abi_type).collect();
		let input_value_names: Vec<_> = inputs
			.iter()
			.map(|input| input.slot_names[0].to_string())
			.collect();

		let mut unique_inputs = Vec::new();

		for &ty in &input_tys {
			if !unique_inputs.contains(&ty) {
				unique_inputs.push(ty);
			}
		}

		let mut required_embeds = Vec::new();

		if let JsBinding::Embed(name) = binding {
			required_embeds.push(quote_spanned!(span=> (#crate_, #name)));
		}

		for ty in &unique_inputs {
			required_embeds.push(quote_spanned!(span=> #r#macro::js_input_embed::<#ty>()));
		}

		if let Some(ty) = output_ty {
			required_embeds.push(quote_spanned!(span=> #r#macro::js_output_embed::<#ty>()));
			required_embeds.push(quote_spanned!(span=> #r#macro::js_result_embed::<#ty>()));
		}

		let input_names_joined = input_value_names.iter().join(", ");
		let js = match binding {
			JsBinding::Generate(binding) => {
				let expression = binding.expression(namespace, &input_value_names);

				let (direct_wrapper, direct_call, indirect_call) =
					if binding.requires_wrapper(namespace) {
						(true, expression.clone(), expression)
					} else {
						let path = binding.path(namespace, &input_value_names);
						(false, path.clone(), format!("{path}({input_names_joined})"))
					};

				Some(quote_spanned! {span=>
					#r#macro::ImportJs {
						direct_wrapper: #direct_wrapper,
						direct_call: #direct_call,
						indirect_call: #indirect_call,
						required_embeds: &[#(#required_embeds),*],
					}
				})
			}
			JsBinding::Embed(name) => {
				let path = format!("this.#jsEmbed.{crate_}['{name}']");
				let indirect_call = format!("{path}({input_names_joined})");

				Some(quote_spanned! {span=>
					#r#macro::ImportJs {
						direct_wrapper: false,
						direct_call: #path,
						indirect_call: #indirect_call,
						required_embeds: &[#(#required_embeds),*],
					}
				})
			}
			JsBinding::Import => None,
		};
		let js_value = if let Some(js) = &js {
			quote_spanned!(span=> ::core::option::Option::Some(#js))
		} else {
			quote_spanned!(span=> ::core::option::Option::None)
		};
		let output = if let Some(output) = output_ty {
			quote_spanned!(span=>
				::core::option::Option::Some(#r#macro::import_output::<#output>())
			)
		} else {
			quote_spanned!(span=> ::core::option::Option::None)
		};
		let has_js = js.is_some();
		let descriptor = quote_spanned! {span=>
			#r#macro::ImportDescriptor::new(
				#crate_,
				#import_name,
				#foreign_name,
				&[#(#input_descriptors),*],
				#output,
				#js_value,
			)
		};

		FunctionImport {
			attrs: attrs.to_vec(),
			descriptor,
			has_js,
			r#macro: r#macro.clone(),
		}
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
			&& binding.owner().is_some()
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

	#[cfg(feature = "macro")]
	fn has_receiver(&self) -> bool {
		matches!(&self.target, JsTarget::Instance(_))
	}

	#[cfg(feature = "macro")]
	fn requires_wrapper(&self, namespace: Option<&str>) -> bool {
		namespace.is_some()
			|| !matches!(&self.target, JsTarget::Global)
			|| !matches!(self.operation, JsOperation::Call)
			|| self.variadic
	}

	#[cfg(feature = "macro")]
	fn path(&self, namespace: Option<&str>, inputs: &[String]) -> String {
		let name = self.property_name.as_ref().unwrap_or(&self.js_name);

		match &self.target {
			JsTarget::Global => Self::global_path(namespace, name),
			JsTarget::Instance(_) => format!("{}.{name}", inputs[0]),
			JsTarget::Static(_) if matches!(self.operation, JsOperation::Construct) => {
				Self::global_path(namespace, name)
			}
			JsTarget::Static(_) => {
				Self::global_path(namespace, &format!("{}.{name}", self.owner_name()))
			}
		}
	}

	#[cfg(feature = "macro")]
	fn global_path(namespace: Option<&str>, name: &str) -> String {
		if let Some(namespace) = namespace {
			format!("globalThis.{namespace}.{name}")
		} else {
			format!("globalThis.{name}")
		}
	}

	#[cfg(feature = "macro")]
	fn expression(&self, namespace: Option<&str>, inputs: &[String]) -> String {
		let path = self.path(namespace, inputs);
		let inputs = if self.has_receiver() {
			&inputs[1..]
		} else {
			inputs
		};
		let call_inputs = if self.variadic {
			let (last, inputs) = inputs
				.split_last()
				.expect("variadic bindings always have an argument");

			if inputs.is_empty() {
				format!("...{last}")
			} else {
				format!("{}, ...{last}", inputs.iter().join(", "))
			}
		} else {
			inputs.iter().join(", ")
		};

		match self.operation {
			JsOperation::Call => format!("{path}({call_inputs})"),
			JsOperation::Construct => format!("new {path}({call_inputs})"),
			JsOperation::Getter => path,
			JsOperation::Setter => format!("{path} = {call_inputs}"),
		}
	}
}

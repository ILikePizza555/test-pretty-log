// Copyright (C) 2019-2023 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

extern crate proc_macro;

use std::iter::Peekable;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as Tokens;

use quote::{quote, ToTokens};

use syn::Attribute;
use syn::ExprLit;
use syn::LitBool;
use syn::LitStr;
use syn::parse_macro_input;
use syn::Expr;
use syn::ItemFn;
use syn::Lit;
use syn::Meta;
use syn::punctuated::Punctuated;
use syn::token::Comma;

#[derive(Debug, Default)]
struct MacroArgs {
  inner_test: Option<Tokens>,
  default_log_filter: Option<String>,
  color: Option<bool>
}

impl MacroArgs {
  fn from_punctuated(punctuated: Punctuated<Meta, Comma>) -> syn::Result<Self> {
    let mut new_self = Self::default();
    let mut punctuated_iter = punctuated.into_iter().peekable();
    
    new_self.parse_inner_test(&mut punctuated_iter);
    new_self.parse_name_value_args(&mut punctuated_iter)?;

    Ok(new_self)
  }

  fn parse_inner_test<I: Iterator<Item = Meta>>(&mut self, punctuated: &mut Peekable<I>) {
    if let Some(Meta::Path(_)) = punctuated.peek() {
      self.inner_test = punctuated.next().map(|path| path.into_token_stream());
    } else {
      self.inner_test = None
    }
  }

  fn parse_name_value_args<I: Iterator<Item = Meta>>(&mut self, punctuated: &mut I) -> syn::Result<()> {
    for m in punctuated {
      let name_value =  m.require_name_value().map_err(map_name_value_error)?;
      let ident = name_value.path.require_ident().map_err(map_name_value_error)?;
      match ident.to_string().as_str() {
        "default_log_filter" => self.default_log_filter = Some(require_lit_str(&name_value.value)?.value()),
        "color" => self.color = Some(require_lit_bool(&name_value.value)?.value()),
        _ => return Err(syn::Error::new_spanned(
          &name_value.path,
          "Unrecognized attribute, see documentation for details.",
        ))
      };
    }

    Ok(())
  }
}


fn map_name_value_error(err: syn::Error) -> syn::Error {
  syn::Error::new(err.span(), "Expected NameValue syntax, e.g. 'default_log_filter = \"debug\"'.")
}

fn require_lit_str(expr: &Expr) -> syn::Result<&LitStr> {
  match expr {
    Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) => Ok(lit_str),
    _ => Err(syn::Error::new_spanned(
      &expr,
      "Failed to parse value, expected a string",
    ))
  }
}

fn require_lit_bool(expr: &Expr) -> syn::Result<&LitBool> {
  match expr {
      Expr::Lit(ExprLit { lit: Lit::Bool(lit_bool), .. }) => Ok(lit_bool),
      _ => Err(syn::Error::new_spanned(
        &expr,
        "Failed to parse value, expected a bool",
      ))
  }
}


#[proc_macro_attribute]
pub fn test(args: TokenStream, item: TokenStream) -> TokenStream {
  let punctuated_args = parse_macro_input!(args with Punctuated::<Meta, syn::Token![,]>::parse_terminated);
  let item = parse_macro_input!(item as ItemFn);

  try_test(punctuated_args, item)
    .unwrap_or_else(syn::Error::into_compile_error)
    .into()
}

fn try_test(punctuated_args: Punctuated<Meta, Comma>, input: ItemFn) -> syn::Result<Tokens> {
  let macro_args = MacroArgs::from_punctuated(punctuated_args)?;

  let ItemFn {
    attrs,
    vis,
    sig,
    block,
  } = input;

  let test_attr = extract_test_attribute(&macro_args, &attrs);

  let env_filter = &macro_args.default_log_filter.map_or(quote! { None }, |s| quote! { Some(#s) });
  let enable_ansi = &macro_args.color.map_or(quote! { None }, |b| quote! { Some(#b) });

  let result = quote! {
    #test_attr
    #(#attrs)*
    #vis #sig {
      let __default_tracing_subscriber_guard = ::test_pretty_log::runtime::init(#env_filter, #enable_ansi);

      #block
    }
  };
  Ok(result)
}


/// Convert macro_args.inner test to a test attribute if not None, otherwise check attrs for
/// a test attribute, if none exist, inject own.
fn extract_test_attribute(macro_args: &MacroArgs, attrs: &Vec<Attribute>) -> Option<Tokens> {
  if let Some(inner_test_arg) = &macro_args.inner_test {
    Some(quote! { #[#inner_test_arg] })
  } else if attrs.iter().find(|&attr| is_test_attribute(attr)).is_none() {
    Some(quote! { #[::core::prelude::v1::test] })
  } else {
    None
  }
}

fn is_test_attribute(attribute: &Attribute) -> bool {
  attribute.meta.path().segments.last().is_some_and(|seg| seg.ident == "test")
}
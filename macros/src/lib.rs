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

  let env_filter = build_env_filter_token_stream(&macro_args);
  let enable_ansi = build_enable_ansi_token_stream(&macro_args);

  let result = quote! {
    #test_attr
    #(#attrs)*
    #vis #sig {
      // We put all initialization code into a separate module here in
      // order to prevent potential ambiguities that could result in
      // compilation errors. E.g., client code could use traits that
      // could have methods that interfere with ones we use as part of
      // initialization; with a `Foo` trait that is implemented for T
      // and that contains a `map` (or similarly common named) method
      // that could cause an ambiguity with `Iterator::map`, for
      // example.
      // The alternative would be to use fully qualified call syntax in
      // all initialization code, but that's much harder to control.
      mod tracing_init {
        use ::test_pretty_log::tracing_subscriber::EnvFilter;
        use ::test_pretty_log::runtime::{parse_env_var_color, init_subscriber};

        pub fn init() {
          init_subscriber(#env_filter, #enable_ansi).expect("Another global subscriber was set");
        }
      }

      tracing_init::init();

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

fn build_env_filter_token_stream(attribute_args: &MacroArgs) -> Tokens {
  match &attribute_args.default_log_filter {
    Some(default_log_filter) => quote! {
      EnvFilter::builder()
        .with_default_directive(
          #default_log_filter
            .parse()
            .expect("test-pretty-log: default_log_filter must be valid"))
        .from_env_lossy()
    },
    _ => quote! { EnvFilter::from_default_env() }
  }
}

fn build_enable_ansi_token_stream(attribute_args: &MacroArgs) -> Tokens {
  match &attribute_args.color {
      Some(color) => color.to_token_stream(),
      None => quote! { parse_env_var_color() }
  }
}
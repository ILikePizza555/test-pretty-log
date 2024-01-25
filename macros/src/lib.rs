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

const ENV_VAR_SPAN_EVENTS: &str = "RUST_LOG_SPAN_EVENTS";
const ENV_VAR_COLOR: &str = "RUST_LOG_COLOR";
const ENV_VAR_FORMAT: &str = "RUST_LOG_FORMAT";

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

  let logging_init = expand_logging_init(&macro_args);
  let tracing_init = expand_tracing_init(&macro_args);

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
      mod init {
        use ::test_pretty_log::runtime::env_var;

        pub fn init() {
          #logging_init
          #tracing_init
        }
      }

      init::init();

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

/// Expand the initialization code for the `log` crate.
#[cfg(feature = "log")]
fn expand_logging_init(attribute_args: &MacroArgs) -> Tokens {
  let add_default_log_filter = if let Some(default_log_filter) = &attribute_args.default_log_filter
  {
    quote! {
      let env_logger_builder = env_logger_builder
        .parse_env(::test_pretty_log::env_logger::Env::default().default_filter_or(#default_log_filter));
    }
  } else {
    quote! {}
  };

  quote! {
    {
      let mut env_logger_builder = ::test_pretty_log::env_logger::builder();
      #add_default_log_filter
      let _ = env_logger_builder.is_test(true).try_init();
    }
  }
}

#[cfg(not(feature = "log"))]
fn expand_logging_init(_attribute_args: &MacroArgs) -> Tokens {
  quote! {}
}

/// Expand the initialization code for the `tracing` crate.
#[cfg(feature = "trace")]
fn expand_tracing_init(attribute_args: &MacroArgs) -> Tokens {
  let env_filter = build_env_filter_token_stream(attribute_args);
  let enable_ansi = build_enable_ansi_token_stream(attribute_args);

  quote! {
    {
      let __internal_event_filter = {
        use ::test_pretty_log::tracing_subscriber::fmt::format::FmtSpan;

        match env_var(#ENV_VAR_SPAN_EVENTS).as_deref() {
          Some(mut value) => {
            value
              .split(",")
              .map(|filter| match filter.trim() {
                "new" => FmtSpan::NEW,
                "enter" => FmtSpan::ENTER,
                "exit" => FmtSpan::EXIT,
                "close" => FmtSpan::CLOSE,
                "active" => FmtSpan::ACTIVE,
                "full" => FmtSpan::FULL,
                _ => panic!("test-pretty-log: RUST_LOG_SPAN_EVENTS must contain filters separated by `,`.\n\t\
                  For example: `active` or `new,close`\n\t\
                  Supported filters: new, enter, exit, close, active, full\n\t\
                  Got: {}", value),
              })
              .fold(FmtSpan::NONE, |acc, filter| filter | acc)
          },
          None => FmtSpan::NONE,
        }
      };

      let subscriber_builder = ::test_pretty_log::tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(#env_filter)
        .with_span_events(__internal_event_filter)
        .with_test_writer()
        .with_ansi(#enable_ansi);

      let _ = match env_var(#ENV_VAR_FORMAT).as_deref() {
        None | Some("pretty") => subscriber_builder.pretty().try_init(),
        Some("full") => subscriber_builder.try_init(),
        Some("compact") => subscriber_builder.compact().try_init(),
        Some(e) => panic!("test-pretty-log: RUST_LOG_FORMAT must be one of `pretty`, `full`, or `compact`. Got: {}", e),
      };
    }
  }
}

#[cfg(feature = "trace")]
fn build_env_filter_token_stream(attribute_args: &MacroArgs) -> Tokens {
  match &attribute_args.default_log_filter {
    Some(default_log_filter) => quote! {
      ::test_pretty_log::tracing_subscriber::EnvFilter::builder()
        .with_default_directive(
          #default_log_filter
            .parse()
            .expect("test-pretty-log: default_log_filter must be valid"))
        .from_env_lossy()
    },
    _ => quote! { ::test_pretty_log::tracing_subscriber::EnvFilter::from_default_env() }
  }
}

#[cfg(feature = "trace")]
fn build_enable_ansi_token_stream(attribute_args: &MacroArgs) -> Tokens {
  match &attribute_args.color {
      Some(color) => color.to_token_stream(),
      None => quote! {
        match env_var(#ENV_VAR_COLOR).as_deref() {
          None | Some("1" | "true" | "t" | "on") => true,
          Some("0" | "false" | "f" | "off") => false,
          Some(_) => panic!("test-pretty-log: {} must be a boolean value", #ENV_VAR_COLOR)
        }
      }
  }
}

#[cfg(not(feature = "trace"))]
fn expand_tracing_init(_attribute_args: &MacroArgs) -> Tokens {
  quote! {}
}

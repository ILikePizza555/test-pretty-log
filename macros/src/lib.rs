// Copyright (C) 2019-2023 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as Tokens;

use quote::quote;

use syn::ExprLit;
use syn::LitBool;
use syn::LitStr;
use syn::parse::Parse;
use syn::parse_macro_input;
use syn::Attribute;
use syn::Expr;
use syn::ItemFn;
use syn::Lit;
use syn::Meta;


#[proc_macro_attribute]
pub fn test(attr: TokenStream, item: TokenStream) -> TokenStream {
  let item = parse_macro_input!(item as ItemFn);
  try_test(attr, item)
    .unwrap_or_else(syn::Error::into_compile_error)
    .into()
}

fn parse_attrs(attrs: Vec<Attribute>) -> syn::Result<(AttributeArgs, Vec<Attribute>)> {
  let mut attribute_args = AttributeArgs::default();
  let mut ignored_attrs = vec![];
  for attr in attrs {
    let matched = attribute_args.try_parse_attr_single(&attr)?;
    // Keep only attrs that didn't match the #[test_log(_)] syntax.
    if !matched {
      ignored_attrs.push(attr);
    }
  }

  Ok((attribute_args, ignored_attrs))
  
}

fn try_test(attr: TokenStream, input: ItemFn) -> syn::Result<Tokens> {
  let inner_test = if attr.is_empty() {
    quote! { ::core::prelude::v1::test }
  } else {
    attr.into()
  };

  let ItemFn {
    attrs,
    vis,
    sig,
    block,
  } = input;

  let (attribute_args, ignored_attrs) = parse_attrs(attrs)?;
  let logging_init = expand_logging_init(&attribute_args);
  let tracing_init = expand_tracing_init(&attribute_args);

  let result = quote! {
    #[#inner_test]
    #(#ignored_attrs)*
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


#[derive(Debug)]
struct AttributeArgs {
  default_log_filter: Option<String>,
  ansi: bool
}

impl Default for AttributeArgs {
    fn default() -> Self {
        Self { 
          default_log_filter: Default::default(),
          ansi: true
        }
    }
}

impl AttributeArgs {
  fn try_parse_attr_single(&mut self, attr: &Attribute) -> syn::Result<bool> {
    let nested_meta = attr.parse_args_with(Meta::parse)?;
    let name_value = nested_meta.require_name_value().map_err(map_name_value_error)?;
    let ident = name_value.path.require_ident().map_err(map_name_value_error)?;

    match ident.to_string().as_str() {
      "default_log_filter" => self.default_log_filter = Some(require_lit_str(&name_value.value)?.value()),
      "ansi" => self.ansi = require_lit_bool(&name_value.value)?.value(),
      _ => return Err(syn::Error::new_spanned(
        &name_value.path,
        "Unrecognized attribute, see documentation for details.",
      ))
    };

    Ok(true)
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


/// Expand the initialization code for the `log` crate.
#[cfg(feature = "log")]
fn expand_logging_init(attribute_args: &AttributeArgs) -> Tokens {
  let add_default_log_filter = if let Some(default_log_filter) = &attribute_args.default_log_filter
  {
    quote! {
      let env_logger_builder = env_logger_builder
        .parse_env(::test_log::env_logger::Env::default().default_filter_or(#default_log_filter));
    }
  } else {
    quote! {}
  };

  quote! {
    {
      let mut env_logger_builder = ::test_log::env_logger::builder();
      #add_default_log_filter
      let _ = env_logger_builder.is_test(true).try_init();
    }
  }
}

#[cfg(not(feature = "log"))]
fn expand_logging_init(_attribute_args: &AttributeArgs) -> Tokens {
  quote! {}
}

/// Expand the initialization code for the `tracing` crate.
#[cfg(feature = "trace")]
fn expand_tracing_init(attribute_args: &AttributeArgs) -> Tokens {
  let env_filter = if let Some(default_log_filter) = &attribute_args.default_log_filter {
    quote! {
      ::test_log::tracing_subscriber::EnvFilter::builder()
        .with_default_directive(
          #default_log_filter
            .parse()
            .expect("test-log: default_log_filter must be valid")
        )
        .from_env_lossy()
    }
  } else {
    quote! { ::test_log::tracing_subscriber::EnvFilter::from_default_env() }
  };

  let enable_ansi = attribute_args.ansi;

  quote! {
    {
      let __internal_event_filter = {
        use ::test_log::tracing_subscriber::fmt::format::FmtSpan;

        match ::std::env::var_os("RUST_LOG_SPAN_EVENTS") {
          Some(mut value) => {
            value.make_ascii_lowercase();
            let value = value.to_str().expect("test-log: RUST_LOG_SPAN_EVENTS must be valid UTF-8");
            value
              .split(",")
              .map(|filter| match filter.trim() {
                "new" => FmtSpan::NEW,
                "enter" => FmtSpan::ENTER,
                "exit" => FmtSpan::EXIT,
                "close" => FmtSpan::CLOSE,
                "active" => FmtSpan::ACTIVE,
                "full" => FmtSpan::FULL,
                _ => panic!("test-log: RUST_LOG_SPAN_EVENTS must contain filters separated by `,`.\n\t\
                  For example: `active` or `new,close`\n\t\
                  Supported filters: new, enter, exit, close, active, full\n\t\
                  Got: {}", value),
              })
              .fold(FmtSpan::NONE, |acc, filter| filter | acc)
          },
          None => FmtSpan::NONE,
        }
      };

      let _ = ::test_log::tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(#env_filter)
        .with_span_events(__internal_event_filter)
        .with_test_writer()
        .with_ansi(#enable_ansi)
        .try_init();
    }
  }
}

#[cfg(not(feature = "trace"))]
fn expand_tracing_init(_attribute_args: &AttributeArgs) -> Tokens {
  quote! {}
}

use std::env::var_os;

use tracing::subscriber::DefaultGuard;
use tracing::{subscriber, Subscriber};
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;

const ENV_VAR_SPAN_EVENTS: &str = "RUST_LOG_SPAN_EVENTS";
const ENV_VAR_COLOR: &str = "RUST_LOG_COLOR";
const ENV_VAR_FORMAT: &str = "RUST_LOG_FORMAT";

/// Gets the value of an environment variable. Panics if the value is not valud UTF-8, returns None if the environment variable isn't set.
fn env_var(key: &str) -> Option<String> {
  let err_msg = format!("test-pretty-log: {} must be valid UTF-8", key);
  var_os(key).map(|oss| oss.to_ascii_lowercase().into_string().expect(&err_msg))
}

fn build_event_filter() -> FmtSpan {
  match env_var(ENV_VAR_SPAN_EVENTS).as_deref() {
    Some(value) => value
      .split(",")
      .map(|filter| match filter.trim() {
        "new" => FmtSpan::NEW,
        "enter" => FmtSpan::ENTER,
        "exit" => FmtSpan::EXIT,
        "close" => FmtSpan::CLOSE,
        "active" => FmtSpan::ACTIVE,
        "full" => FmtSpan::FULL,
        _ => panic!(
          "test-pretty-log: RUST_LOG_SPAN_EVENTS must contain filters separated by `,`.\n\t\
                For example: `active` or `new,close`\n\t\
                Supported filters: new, enter, exit, close, active, full\n\t\
                Got: {}",
          value
        ),
      })
      .fold(FmtSpan::NONE, |acc, filter| filter | acc),
    None => FmtSpan::NONE,
  }
}

/// Parses the value of `RUST_LOG_COLOR`.
/// If the environment variable isn't set, it returns the default value (true)
/// Panics if the environment variable isn't valid UTF-8 or not a boolean value.
fn parse_env_var_color() -> bool {
  match env_var(ENV_VAR_COLOR).as_deref() {
    None | Some("1" | "true" | "t" | "on") => true,
    Some("0" | "false" | "f" | "off") => false,
    Some(_) => panic!("test-pretty-log: {} must be a boolean value", ENV_VAR_COLOR)
  }
}

fn build_env_filter(default_log_filter: Option<&str>) -> EnvFilter {
  match default_log_filter {
      Some(directive) => EnvFilter::builder()
        .with_default_directive(directive.parse().expect("test-pretty-log: default_log_filter must be valid"))
        .from_env_lossy(),
      None => EnvFilter::from_default_env()
  }
}

pub fn init(env_filter_arg: Option<&str>, with_ansi_arg: Option<bool>) -> DefaultGuard {
  let __internal_event_filter = build_event_filter();
  let env_filter = build_env_filter(env_filter_arg);
  let with_ansi = with_ansi_arg.unwrap_or_else(parse_env_var_color);

  let subscriber_builder = fmt()
    .with_env_filter(env_filter)
    .with_span_events(__internal_event_filter)
    .with_test_writer()
    .with_ansi(with_ansi);

  let subscriber: Box<dyn Subscriber + Send + Sync> = match env_var(ENV_VAR_FORMAT).as_deref() {
    None | Some("pretty") => Box::new(subscriber_builder.pretty().finish()),
    Some("full") => Box::new(subscriber_builder.finish()),
    Some("compact") => Box::new(subscriber_builder.compact().finish()),
    Some(e) => panic!("test-pretty-log: RUST_LOG_FORMAT must be one of `pretty`, `full`, or `compact`. Got: {}", e),
  };

  subscriber::set_default(subscriber)
}
use std::env::var_os;
use std::error::Error;

use tracing_subscriber::fmt;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;

const ENV_VAR_SPAN_EVENTS: &str = "RUST_LOG_SPAN_EVENTS";
const ENV_VAR_COLOR: &str = "RUST_LOG_COLOR";
const ENV_VAR_FORMAT: &str = "RUST_LOG_FORMAT";

/// Gets the value of an environment variable. Panics if the value is not valud UTF-8, returns None if the environment variable isn't set.
pub fn env_var(key: &str) -> Option<String> {
  let err_msg = format!("test-pretty-log: {} must be valid UTF-8", key);
  var_os(key).map(|oss| oss.to_ascii_lowercase().into_string().expect(&err_msg))
}

pub fn build_event_filter() -> FmtSpan {
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
pub fn parse_env_var_color() -> bool {
  match env_var(ENV_VAR_COLOR).as_deref() {
    None | Some("1" | "true" | "t" | "on") => true,
    Some("0" | "false" | "f" | "off") => false,
    Some(_) => panic!("test-pretty-log: {} must be a boolean value", ENV_VAR_COLOR)
  }
}

pub fn init_subscriber(env_filter: EnvFilter, with_ansi: bool) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
  let __internal_event_filter = build_event_filter();

  let subscriber_builder = fmt()
    .with_env_filter(env_filter)
    .with_span_events(__internal_event_filter)
    .with_test_writer()
    .with_ansi(with_ansi);

  match env_var(ENV_VAR_FORMAT).as_deref() {
    None | Some("pretty") => subscriber_builder.pretty().try_init(),
    Some("full") => subscriber_builder.try_init(),
    Some("compact") => subscriber_builder.compact().try_init(),
    Some(e) => panic!("test-pretty-log: RUST_LOG_FORMAT must be one of `pretty`, `full`, or `compact`. Got: {}", e),
  }
}
test-pretty-log
========


**test-pretty-log** is a crate that takes care of automatically initializing
logging and/or tracing for Rust tests.
 
It is based off [test-log](https://github.com/d-e-s-o/test-log) and enables
the logs to use pretty colors! :3

Usage
-----

The crate provides a custom `#[test]` attribute that, when used for
running a particular test, takes care of initializing `log` and/or
`tracing` beforehand.

#### Example

As such, usage is as simple as importing and using said attribute:
```rust
use test_pretty_log::test;

#[test]
fn it_works() {
  info!("Checking whether it still works...");
  assert_eq!(2 + 2, 4);
  info!("Looks good!");
}
```

It is of course also possible to initialize logging for a chosen set of
tests, by only annotating these with the custom attribute:
```rust
#[test_pretty_log::test]
fn it_still_works() {
  // ...
}
```

You can also stack another attribute. For example, suppose you use
[`#[tokio::test]`][tokio-test] to run async tests:
```rust
use test_log::test;

#[test]
#[tokio::test]
async fn it_still_works() {
  // ...
}
```

Lastly, you can disable coloring for a test with a parameter:
```rust
use test_log::test;

#[test(ansi=false)]
fn no_more_colored_output() {
  // :blobfoxsad:
}
```

#### Logging Configuration

As usual when running `cargo test`, the output is captured by the
framework by default and only shown on test failure. The `--nocapture`
argument can be supplied in order to overwrite this setting. E.g.,
```bash
$ cargo test -- --nocapture
```

Furthermore, the `RUST_LOG` environment variable is honored and can be
used to influence the log level to work with (among other things).
Please refer to the [`env_logger` docs][env-docs-rs] for more
information.

If the `trace` feature is enabled, the `RUST_LOG_SPAN_EVENTS`
environment variable can be used to configure the tracing subscriber to
log synthesized events at points in the span lifecycle. Set the variable
to a comma-separated list of events you want to see. For example,
`RUST_LOG_SPAN_EVENTS=full` or `RUST_LOG_SPAN_EVENTS=new,close`.

Valid events are `new`, `enter`, `exit`, `close`, `active`, and `full`.
See the [`tracing_subscriber` docs][tracing-events-docs-rs] for details
on what the events mean.

#### Features

The crate comes with two features:
- `log`, enabled by default, controls initialization for the `log`
  crate.
- `trace`, disabled by default, controls initialization for the
  `tracing` crate.

Depending on what backend the crate-under-test (and its dependencies)
use, the respective feature should be enabled to make messages that are
emitted by the test manifest on the console.

[docs-rs]: https://docs.rs/crate/test-log
[env-docs-rs]: https://docs.rs/env_logger/0.10.1/env_logger
[log]: https://crates.io/crates/log
[tokio-test]: https://docs.rs/tokio/1.4.0/tokio/attr.test.html
[tracing]: https://crates.io/crates/tracing
[tracing-events-docs-rs]: https://docs.rs/tracing-subscriber/0.3.1/tracing_subscriber/fmt/struct.SubscriberBuilder.html#method.with_span_events

use tracing::{info, debug};

/// Manually verify these tests with the following command
/// ```sh
/// RUST_LOG=debug cargo test --test color --features trace -- --nocapture
/// ```

#[test_pretty_log::test(color = false)]
fn trace_with_color_off() {
  info!("This should NOT be colored");
  assert!(true);
  debug!("done");
}

#[test_pretty_log::test]
fn trace_with_color() {
  info!("This SHOULD be colored");
  assert!(true);
  debug!("done");
}
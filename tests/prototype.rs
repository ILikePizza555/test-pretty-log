//! Use this file for iterating on the derive code. You can view the
//! expanded code for any given configuration by updating this file and
//! running:
//!
//! ```sh
//! cargo expand --test=prototype
//! ```

use tracing::debug;

#[test_pretty_log::test()]
fn it_works() {
  debug!("test");
  assert_eq!(2 + 2, 4);
}
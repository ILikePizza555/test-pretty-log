//! Use this file for iterating on the derive code. You can view the
//! expanded code for any given configuration by updating this file and
//! running:
//!
//! ```sh
//! cargo expand --test=prototype
//! ```


#[test_pretty_log::test(color = false)]
fn it_works() {
  assert_eq!(2 + 2, 4);
}

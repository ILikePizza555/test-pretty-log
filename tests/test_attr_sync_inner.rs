#[test_pretty_log::test(test)]
fn without_return_type() {
  assert_eq!(2 + 2, 4);
}

#[test_pretty_log::test(test)]
fn with_return_type() -> Result<(), String> {
  Ok(())
}

#[test_pretty_log::test(test)]
#[should_panic(expected = "success")]
fn with_panic() {
  panic!("success")
}

#[test_case::test_case(-2, -4)]
#[test_pretty_log::test(test)]
fn with_test_args(x: i8, y: i8) {
  assert_eq!(x, -2);
  assert_eq!(y, -4);
}

#[test_case::test_case(-2, -4; "my test name")]
#[test_pretty_log::test(test)]
fn with_test_args_and_name(x: i8, y: i8) {
  assert_eq!(x, -2);
  assert_eq!(y, -4);
}

#[should_panic]
#[test_case::test_case(-2, -4; "my test name")]
#[test_pretty_log::test(test)]
fn with_test_args_and_name_and_panic(x: i8, y: i8) {
  assert_eq!(x, 0);
}
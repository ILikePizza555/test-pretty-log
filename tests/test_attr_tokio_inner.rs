#[test_pretty_log::test(tokio::test)]
async fn without_return_type() {
  assert_eq!(async { 2 + 2 }.await, 4);
}

#[test_pretty_log::test(tokio::test)]
async fn with_return_type() -> Result<(), String> {
  Ok(async { () }.await)
}

#[test_pretty_log::test(tokio::test)]
#[should_panic(expected = "success")]
async fn with_panic() {
  panic!("success")
}

#[test_case::test_case(-2, -4)]
#[test_pretty_log::test(tokio::test)]
async fn with_test_args(x: i8, y: i8) {
  assert_eq!(async { x }.await, -2);
  assert_eq!(async { y }.await, -4);
}

#[test_case::test_case(-2, -4; "my test name")]
#[test_pretty_log::test(tokio::test)]
async fn with_test_args_and_name(x: i8, y: i8) {
  assert_eq!(async { x }.await, -2);
  assert_eq!(async { y }.await, -4);
}

#[should_panic]
#[test_case::test_case(-2, -4; "my test name")]
#[test_pretty_log::test(tokio::test)]
async fn with_test_args_and_name_and_panic(x: i8, _y: i8) {
  assert_eq!(x, 0);
}
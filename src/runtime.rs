use std::env::var_os;

pub fn env_var(key: &str) -> Option<String> {
    let err_msg = format!("test-pretty-log: {} must be valid UTF-8", key);
    var_os(key).map(
        |oss| oss
            .to_ascii_lowercase()
            .into_string()
            .expect(&err_msg))
}
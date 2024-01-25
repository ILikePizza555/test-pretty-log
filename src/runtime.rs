use std::env::var_os;

pub fn env_var(key: &str) -> Option<String> {
    var_os(key).map(
        |oss| oss
            .to_ascii_lowercase()
            .to_str()
            .expect(format!("test-pretty-log: {} must be valid UTF-8", key).as_str())
            .to_owned())
}
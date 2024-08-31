use std::env;

use serde::Deserialize;
use serde_json::Error;

pub fn get_env_var(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("Missing environment variable: {}", key))
}

pub fn deserialize_message_str<T: for<'a> Deserialize<'a>>(data: &str) -> Result<T, Error> {
    serde_json::from_str(data)
}

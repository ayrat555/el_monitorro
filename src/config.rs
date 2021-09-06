use std::fmt::Debug;
use std::{env, str::FromStr};

pub struct Config {}

impl Config {
    pub fn database_url() -> String {
        Self::read_var("DATABASE_URL")
    }

    pub fn telegram_bot_token() -> String {
        Self::read_var("TELEGRAM_BOT_TOKEN")
    }

    pub fn request_timeout_in_seconds() -> u64 {
        Self::read_var_with_default("REQUEST_TIMEOUT", "5")
    }

    pub fn owner_telegram_id() -> Option<i64> {
        Self::read_var_option("OWNER_TELEGRAM_ID")
    }

    pub fn telegram_bot_handle() -> String {
        Self::read_var_with_default("TELEGRAM_BOT_HANDLE", "")
    }

    pub fn subscription_limit() -> i64 {
        Self::read_var_with_default("SUBSCRIPTION_LIMIT", "20")
    }

    fn read_var_with_default<T: FromStr + Debug>(name: &str, default_value: &str) -> T
    where
        <T as FromStr>::Err: std::fmt::Debug,
    {
        let value = env::var(name).unwrap_or_else(|_| default_value.to_string());

        value.parse().unwrap()
    }

    fn read_var<T: FromStr + Debug>(name: &str) -> T
    where
        <T as FromStr>::Err: std::fmt::Debug,
    {
        let value = env::var(name).unwrap_or_else(|_| panic!("{} must be set", name));

        value.parse().unwrap()
    }

    fn read_var_option<T: FromStr + Debug>(name: &str) -> Option<T>
    where
        <T as FromStr>::Err: std::fmt::Debug,
    {
        match env::var(name) {
            Ok(val) => {
                let parsed_value: T = val.parse().unwrap();
                Some(parsed_value)
            }
            Err(_error) => None,
        }
    }
}

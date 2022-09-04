use std::fmt::Debug;
use std::{env, str::FromStr};

// 3 days as much
const MAX_SECONDS: u32 = 259_200;

pub struct Config {}

impl Config {
    pub fn database_url() -> String {
        Self::read_var("DATABASE_URL")
    }

    pub fn telegram_base_url() -> String {
        Self::read_var_with_default("TELEGRAM_BASE_URL", "https://api.telegram.org/bot")
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

    pub fn admin_telegram_id() -> Option<i64> {
        Self::read_var_option("ADMIN_TELEGRAM_ID")
    }

    pub fn telegram_bot_handle() -> String {
        Self::read_var_with_default("TELEGRAM_BOT_HANDLE", "")
    }

    pub fn deliver_workers_number() -> u32 {
        Self::read_var_with_default("DELIVER_WORKERS_NUMBER", "1")
    }

    pub fn sync_workers_number() -> u32 {
        Self::read_var_with_default("SYNC_WORKERS_NUMBER", "1")
    }

    pub fn clean_workers_number() -> u32 {
        Self::read_var_with_default("CLEAN_WORKERS_NUMBER", "1")
    }

    pub fn subscription_limit() -> i64 {
        Self::read_var_with_default("SUBSCRIPTION_LIMIT", "20")
    }

    pub fn commands_db_pool_number() -> u32 {
        Self::read_var_with_default("DATABASE_POOL_SIZE", "5")
    }

    fn check_interval(interval: &u32) {
        if !(1..=MAX_SECONDS).contains(interval) {
            panic!(
                "Value {} is not in the interval [1 second , MAX_SECONDS]",
                interval
            )
        }
    }

    pub fn deliver_interval_in_seconds() -> u32 {
        let interval: u32 = Self::read_var_with_default("DELIVER_INTERVAL_SECONDS", "60");

        Self::check_interval(&interval);
        interval
    }

    pub fn deliver_cron_pattern() -> String {
        let interval = Config::deliver_interval_in_seconds();
        seconds_to_cron(interval)
    }

    pub fn sync_interval_in_seconds() -> u32 {
        let interval = Self::read_var_with_default("SYNC_INTERVAL_SECONDS", "60");

        Self::check_interval(&interval);

        interval
    }

    pub fn sync_cron_pattern() -> String {
        let interval = Config::sync_interval_in_seconds();
        seconds_to_cron(interval)
    }

    pub fn clean_interval_in_seconds() -> u32 {
        let interval = Self::read_var_with_default("CLEAN_INTERVAL_SECONDS", "3600");

        Self::check_interval(&interval);

        interval
    }

    pub fn clean_cron_pattern() -> String {
        let interval = Config::clean_interval_in_seconds();
        seconds_to_cron(interval)
    }

    pub fn all_binaries() -> bool {
        Self::read_var_option::<String>("ALL_BINARIES").is_some()
    }

    fn read_var_with_default<T: FromStr + Debug>(name: &str, default_value: &str) -> T
    where
        <T as FromStr>::Err: std::fmt::Debug,
    {
        let value = env::var(name).unwrap_or_else(|_| default_value.to_string());

        value
            .parse()
            .unwrap_or_else(|_| panic!("{} can not be parsed", name))
    }

    fn read_var<T: FromStr + Debug>(name: &str) -> T
    where
        <T as FromStr>::Err: std::fmt::Debug,
    {
        let value = env::var(name).unwrap_or_else(|_| panic!("{} must be set", name));

        value
            .parse()
            .unwrap_or_else(|_| panic!("{} can not be parsed", name))
    }

    fn read_var_option<T: FromStr + Debug>(name: &str) -> Option<T>
    where
        <T as FromStr>::Err: std::fmt::Debug,
    {
        match env::var(name) {
            Ok(val) => {
                let parsed_value: T = val
                    .parse()
                    .unwrap_or_else(|_| panic!("{} can not be parsed", name));
                Some(parsed_value)
            }
            Err(_error) => None,
        }
    }
}

pub fn seconds_to_cron(seconds_amount: u32) -> String {
    let vec = seconds_to_units(seconds_amount);

    match vec.len() {
        1 => format!("*/{} * * * * * *", vec[0]),
        2 => format!("{} */{} * * * * *", vec[0], vec[1]),
        3 => format!("{} {} */{} * * * *", vec[0], vec[1], vec[2]),
        4 => format!("{} {} {} */{} * * *", vec[0], vec[1], vec[2], vec[3]),
        _ => panic!("Error fix units for cron"),
    }
}

pub fn seconds_to_units(seconds_amount: u32) -> Vec<u32> {
    let mut vec = vec![];
    let mut unit = seconds_amount;
    let mut finish = false;
    let mut divs = [60u32, 60u32, 24u32].iter();
    let mut div_option = divs.next();

    while div_option.is_some() && !finish {
        let div = *div_option.unwrap();
        if unit < div {
            vec.push(unit);
            finish = true;
        } else {
            vec.push(unit % div);

            unit /= div;
        }
        div_option = divs.next();
    }

    if !finish {
        vec.push(unit);
    }

    vec
}

mod test {

    #[test]
    fn test_second_to_units() {
        let twelve_hours = super::seconds_to_units(43200);
        assert_eq!(vec![0u32, 0u32, 12u32], twelve_hours);

        let one_day_twelve_hours = super::seconds_to_units(43200 * 3);
        assert_eq!(vec![0u32, 0u32, 12u32, 1u32], one_day_twelve_hours);

        let one_day_twelve_hours_twelve_minutes_twelwe_seconds =
            super::seconds_to_units(43200 * 3 + 12 * 61);
        assert_eq!(
            vec![12u32, 12u32, 12u32, 1u32],
            one_day_twelve_hours_twelve_minutes_twelwe_seconds
        );

        let twelve_minutes = super::seconds_to_units(12 * 60);
        assert_eq!(vec![0u32, 12u32], twelve_minutes);

        let twelve_seconds = super::seconds_to_units(12);
        assert_eq!(vec![12u32], twelve_seconds);

        let twelve_minutes_twelve_seconds = super::seconds_to_units(12 * 61);
        assert_eq!(vec![12u32, 12u32], twelve_minutes_twelve_seconds);
    }
}

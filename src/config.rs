use std::env;
use std::sync::LazyLock;

static CONFIG: LazyLock<Config> = LazyLock::new(Config::from_env);

pub struct Config {
    pub filter_out_shorts: bool,
    pub videos_per_page: i64,
    pub feed_refresh_interval_mins: u64,
}

impl Config {
    pub fn get() -> &'static Config {
        &CONFIG
    }

    fn from_env() -> Self {
        Self {
            filter_out_shorts: Self::bool_env_parse("FILTER_OUT_SHORTS", false),
            videos_per_page: Self::env_parse("VIDEOS_PER_PAGE", 24),
            feed_refresh_interval_mins: Self::env_parse("FEED_REFRESH_INTERVAL_MINS", 5),
        }
    }

    fn env_parse<T: std::str::FromStr>(key: &str, default: T) -> T {
        env::var(key)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }

    fn bool_env_parse(key: &str, default: bool) -> bool {
        env::var(key)
            .ok()
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(default)
    }
}

use std::sync::LazyLock;

static CONFIG: LazyLock<Config> = LazyLock::new(Config::from_env);

pub struct Config {
    pub filter_out_shorts: bool,
    pub videos_per_page: i64,
}

impl Config {
    pub fn get() -> &'static Config {
        &CONFIG
    }

    fn from_env() -> Self {
        Self {
            filter_out_shorts: std::env::var("FILTER_OUT_SHORTS")
                .map(|v| v.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
            videos_per_page: std::env::var("VIDEOS_PER_PAGE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(24),
        }
    }
}

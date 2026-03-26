use std::sync::LazyLock;

static CONFIG: LazyLock<Config> = LazyLock::new(Config::from_env);

pub struct Config {
    pub filter_shorts: bool,
}

impl Config {
    pub fn get() -> &'static Config {
        &CONFIG
    }

    fn from_env() -> Self {
        Self {
            filter_shorts: std::env::var("FILTER_SHORTS")
                .map(|v| v.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
        }
    }
}

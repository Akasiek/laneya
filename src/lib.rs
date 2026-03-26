pub mod handlers {
    pub mod channel_api_handler;
    pub mod channel_page_handler;
    pub mod error_handler;
    pub mod index_handler;
    pub mod video_handler;
    pub mod ws_handler;
}
pub mod models {
    pub mod channel;
    pub mod video;
}
pub mod repositories {
    pub mod channel_repository;
    pub mod video_repository;
}
pub mod services {
    pub mod feed_reader_service;
}
pub mod app_state;
pub mod config;
pub mod db;
pub mod macros;
pub mod router;
pub mod scheduler;
pub mod schema;
pub mod templates;
pub mod tracer;
pub mod web_server;

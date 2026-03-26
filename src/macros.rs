#[macro_export]
macro_rules! render_template {
    ($template:expr) => {{
        match $template.render() {
            Ok(html) => axum::response::Html(html),
            Err(_) => $crate::handlers::error_handler::server_error().await,
        }
    }};
}
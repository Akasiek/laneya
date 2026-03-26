use axum::Router;
use axum::extract::MatchedPath;
use axum::http::Request;
use axum::routing::{get};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::info_span;
use crate::app_state::AppState;
use crate::handlers::index_handler::index;
use crate::handlers::ws_handler::ws_handler;

pub fn get_app_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/ws", get(ws_handler))
        .nest_service("/static", ServeDir::new("templates/static"))
        .with_state(state)
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                let matched_path = request
                    .extensions()
                    .get::<MatchedPath>()
                    .map(MatchedPath::as_str);

                info_span!(
                    "http_request",
                    method = ?request.method(),
                    matched_path,
                    some_other_field = tracing::field::Empty,
                )
            }),
        )
}

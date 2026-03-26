use crate::app_state::AppState;
use crate::handlers::channel_api_handler::{
    add_channel, channel_row, delete_channel, edit_channel_form, update_channel,
};
use crate::handlers::channel_page_handler::channels_page;
use crate::handlers::index_handler::index;
use crate::handlers::video_handler::video_list;
use crate::handlers::ws_handler::ws_handler;
use axum::Router;
use axum::extract::MatchedPath;
use axum::http::Request;
use axum::routing::{get, post, put};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::info_span;

pub fn get_app_router(state: AppState) -> Router {
    Router::new()
        .merge(pages_router())
        .nest("/channels", channels_router())
        .nest("/videos", videos_router())
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

/// Routes that render full pages navigated to by the user.
fn pages_router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/channels", get(channels_page))
}

/// Routes used internally by HTMX for fragments and channel mutations.
fn channels_router() -> Router<AppState> {
    Router::new()
        .route("/", post(add_channel))
        .route("/{id}", put(update_channel).delete(delete_channel))
        .route("/{id}/edit", get(edit_channel_form))
        .route("/{id}/row", get(channel_row))
}

/// Routes used internally by HTMX for video fragments.
fn videos_router() -> Router<AppState> {
    Router::new().route("/", get(video_list))
}

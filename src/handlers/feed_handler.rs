use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::app_state::AppState;
use crate::scheduler;

pub async fn refresh_feed_handler(State(state): State<AppState>) -> impl IntoResponse {
    tokio::spawn(async move {
        scheduler::refresh_feed(state.feed_tx).await;
    });
    StatusCode::NO_CONTENT
}


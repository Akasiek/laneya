use askama::Template;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;

use crate::app_state::AppState;
use crate::db;
use crate::repositories::video_repository::VideoRepository;
use crate::templates::VideoGridTemplate;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut rx = state.feed_tx.subscribe();

    // Send initial video grid right after connection
    if let Some(html) = render_video_grid() {
        if socket.send(Message::text(html)).await.is_err() {
            return;
        }
    }

    // Listen for feed updates and push new grid
    while rx.recv().await.is_ok() {
        if let Some(html) = render_video_grid() {
            if socket.send(Message::text(html)).await.is_err() {
                break;
            }
        }
    }
}

fn render_video_grid() -> Option<String> {
    let conn = &mut db::establish_connection();
    let videos = VideoRepository::find_all_recent(conn).ok()?;
    let template = VideoGridTemplate { videos };
    template.render().ok()
}

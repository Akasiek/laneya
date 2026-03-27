use crate::app_state::AppState;
use crate::db;
use crate::repositories::video_repository::VideoRepository;
use crate::templates::VideoGridTemplate;
use askama::Template;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use tokio::sync::broadcast::Receiver;

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut rx = state.feed_tx.subscribe();

    if send_initial_video_grid(&mut socket).await.is_err() {
        return;
    }

    watch_for_feed_updates(&mut socket, &mut rx).await;
}

async fn send_initial_video_grid(socket: &mut WebSocket) -> Result<(), ()> {
    if let Some(html) = render_video_grid() {
        if socket.send(Message::text(html)).await.is_err() {
            return Err(());
        }
    }

    Ok(())
}

async fn watch_for_feed_updates(socket: &mut WebSocket, rx: &mut Receiver<()>) {
    while rx.recv().await.is_ok() {
        let Some(html) = render_video_grid() else {
            continue;
        };

        if socket.send(Message::text(html)).await.is_err() {
            break;
        }
    }
}

fn render_video_grid() -> Option<String> {
    let conn = &mut db::establish_connection();
    let (videos, total_pages) = VideoRepository::find_paginated(conn, 1).ok()?;
    let template = VideoGridTemplate {
        videos,
        page: 1,
        total_pages,
    };
    template.render().ok()
}

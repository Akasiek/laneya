use crate::app_state::AppState;
use crate::db;
use crate::repositories::video_repository::VideoRepository;
use crate::templates::VideoGridTemplate;
use askama::Template;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use std::time::Duration;
use tokio::sync::broadcast::Receiver;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::{Instant, Interval, interval, sleep_until};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(20);
const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(45);

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut rx = state.feed_tx.subscribe();

    if send_video_grid(&mut socket).await.is_err() {
        return;
    }

    watch_for_feed_updates(&mut socket, &mut rx).await;
}

async fn send_video_grid(socket: &mut WebSocket) -> Result<(), ()> {
    if let Some(html) = render_video_grid() {
        if socket.send(Message::text(html)).await.is_err() {
            return Err(());
        }
    }

    Ok(())
}

async fn watch_for_feed_updates(socket: &mut WebSocket, rx: &mut Receiver<()>) {
    let mut heartbeat = interval(HEARTBEAT_INTERVAL);
    let mut last_pong = Instant::now();

    loop {
        if handle_next_socket_event(socket, rx, &mut heartbeat, &mut last_pong)
            .await
            .is_err()
        {
            break;
        }
    }
}

async fn handle_next_socket_event(
    socket: &mut WebSocket,
    rx: &mut Receiver<()>,
    heartbeat: &mut Interval,
    last_pong: &mut Instant,
) -> Result<(), ()> {
    tokio::select! {
        _ = heartbeat.tick() => {
            socket.send(Message::Ping(Vec::new().into())).await.map_err(|_| ())
        }
        _ = sleep_until(*last_pong + HEARTBEAT_TIMEOUT) => {
            tracing::debug!("WebSocket heartbeat timed out");
            Err(())
        }
        update = rx.recv() => {
            match update {
                Ok(()) | Err(RecvError::Lagged(_)) => send_video_grid(socket).await,
                Err(RecvError::Closed) => Err(()),
            }
        }
        message = socket.recv() => {
            match message {
                Some(Ok(Message::Pong(_))) => {
                    *last_pong = Instant::now();
                    Ok(())
                }
                Some(Ok(Message::Close(_))) | Some(Err(_)) | None => Err(()),
                _ => Ok(()),
            }
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

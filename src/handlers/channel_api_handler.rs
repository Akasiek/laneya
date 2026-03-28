use crate::app_state::AppState;
use crate::models::channel::NewChannel;
use crate::render_template;
use crate::repositories::channel_repository::ChannelRepository;
use crate::services::csv_import_service::{extract_csv_bytes, parse_takeout_csv};
use crate::services::{feed_reader_service, ws_service};
use crate::templates::{ChannelEditRowTemplate, ChannelRowTemplate};
use crate::{db, html_error};
use askama::Template;
use axum::Form;
use axum::extract::multipart::Multipart;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Response};
use feed_reader_service::read_channel_feed;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ChannelForm {
    pub channel_id: String,
    pub channel_name: Option<String>,
}

pub async fn add_channel(
    State(state): State<AppState>,
    Form(form): Form<ChannelForm>,
) -> impl IntoResponse {
    let channel_id = form.channel_id.trim().to_string();
    if channel_id.is_empty() {
        return html_error!("Channel ID is required.").into_response();
    }

    let channel_name = match read_channel_feed(&channel_id, None).await {
        Ok(feed) => feed.title,
        Err(e) => {
            return html_error!("{e}").into_response();
        }
    };

    let conn = &mut db::establish_connection();
    match ChannelRepository::create(
        conn,
        NewChannel {
            channel_id,
            channel_name,
        },
    ) {
        Ok(channel) => {
            ChannelRepository::spawn_channel_feed_refresh(state, channel.id);
            htmx_redirect("/channels")
        }
        Err(e) => {
            tracing::error!("Failed to add channel: {}", e);
            html_error!("Error: {e}").into_response()
        }
    }
}

pub async fn update_channel(
    Path(id): Path<i32>,
    State(state): State<AppState>,
    Form(form): Form<ChannelForm>,
) -> Html<String> {
    let channel_id = form.channel_id.trim().to_string();
    let channel_name = form
        .channel_name
        .as_deref()
        .unwrap_or("")
        .trim()
        .to_string();

    let validation_error = read_channel_feed(&channel_id, None).await.err();
    let conn = &mut db::establish_connection();

    if let Some(ref e) = validation_error {
        if let Some(channel) = ChannelRepository::find_by_id(conn, id).unwrap_or_default() {
            let template = ChannelEditRowTemplate {
                channel,
                error: Some(format!("Invalid channel ID: {e}")),
            };
            return render_template!(template);
        }
        return html_error!("{e}");
    }

    match ChannelRepository::update(conn, id, channel_name, channel_id) {
        Ok(_) => {
            ChannelRepository::spawn_channel_feed_refresh(state, id);
            if let Some(channel) = ChannelRepository::find_by_id(conn, id).unwrap_or_default() {
                let template = ChannelRowTemplate { channel };
                render_template!(template)
            } else {
                Html(String::new())
            }
        }
        Err(e) => Html(html_error!("Error: {e}").0),
    }
}

pub async fn delete_channel(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let conn = &mut db::establish_connection();
    match ChannelRepository::delete(conn, id) {
        Ok(_) => {
            ws_service::send_refresh_notification(state.feed_tx).await;
            Html(String::new()).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to delete channel {}: {}", id, e);
            html_error!("Error: {e}").into_response()
        }
    }
}

pub async fn bulk_import_channels(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let csv_bytes = match extract_csv_bytes(&mut multipart).await {
        Ok(b) => b,
        Err(r) => return r,
    };

    let entries = match parse_takeout_csv(&csv_bytes) {
        Ok(e) => e,
        Err(e) => return html_error!("Invalid CSV: {e}").into_response(),
    };

    let conn = &mut db::establish_connection();
    let (imported, skipped) = match ChannelRepository::bulk_create(conn, entries) {
        Ok(result) => result,
        Err(e) => return html_error!("{e}").into_response(),
    };

    if imported == 0 {
        return html_error!("All {skipped} channel(s) are already in your list.").into_response();
    }

    ChannelRepository::spawn_all_feeds_refresh(state).await;
    htmx_redirect("/channels")
}

pub async fn render_edit_channel_form(Path(id): Path<i32>) -> impl IntoResponse {
    let conn = &mut db::establish_connection();
    if let Some(channel) = ChannelRepository::find_by_id(conn, id).unwrap_or_default() {
        let template = ChannelEditRowTemplate {
            channel,
            error: None,
        };
        render_template!(template)
    } else {
        Html("<p class='text-red-500'>Not found</p>".to_string())
    }
}

pub async fn render_channel_row(Path(id): Path<i32>) -> impl IntoResponse {
    let conn = &mut db::establish_connection();
    if let Some(channel) = ChannelRepository::find_by_id(conn, id).unwrap_or_default() {
        let template = ChannelRowTemplate { channel };
        render_template!(template)
    } else {
        Html(String::new())
    }
}

fn htmx_redirect(path: &str) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert("HX-Redirect", path.parse().unwrap());
    (headers, Html(String::new())).into_response()
}

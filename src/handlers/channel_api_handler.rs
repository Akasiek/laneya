use crate::app_state::AppState;
use crate::db;
use crate::models::channel::NewChannel;
use crate::render_template;
use crate::repositories::channel_repository::ChannelRepository;
use crate::services::csv_import_service::parse_takeout_csv;
use crate::services::feed_reader_service::fetch_channel_name;
use crate::templates::{ChannelEditRowTemplate, ChannelRowTemplate};
use askama::Template;
use axum::Form;
use axum::extract::multipart::Multipart;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Response};
use diesel::SqliteConnection;
use serde::Deserialize;
use std::collections::HashSet;
use tracing::info;

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
        return Html(r#"<p class="text-red-500 text-sm">Channel ID is required.</p>"#.to_string())
            .into_response();
    }

    let channel_name = match fetch_channel_name(&channel_id).await {
        Ok(name) => name,
        Err(e) => {
            return Html(format!(r#"<p class="text-red-500 text-sm">{e}</p>"#)).into_response();
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
        Ok(new_id) => {
            spawn_feed_refresh(state, new_id);
            let mut headers = HeaderMap::new();
            headers.insert("HX-Redirect", "/channels".parse().unwrap());
            (headers, Html(String::new())).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to add channel: {}", e);
            Html(format!(r#"<p class="text-red-500 text-sm">Error: {e}</p>"#)).into_response()
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

    let validation_error = fetch_channel_name(&channel_id).await.err();

    let conn = &mut db::establish_connection();

    if let Some(ref e) = validation_error {
        if let Some(channel) = ChannelRepository::find_by_id(conn, id).unwrap_or_default() {
            let template = ChannelEditRowTemplate {
                channel,
                error: Some(e.clone()),
            };
            return render_template!(template);
        }
        return Html(format!(r#"<p class="text-red-500 text-sm">{e}</p>"#));
    }

    match ChannelRepository::update(conn, id, channel_name, channel_id) {
        Ok(_) => {
            spawn_feed_refresh(state, id);
            if let Some(channel) = ChannelRepository::find_by_id(conn, id).unwrap_or_default() {
                let template = ChannelRowTemplate { channel };
                render_template!(template)
            } else {
                Html(String::new())
            }
        }
        Err(e) => Html(format!(r#"<p class="text-red-500 text-sm">Error: {e}</p>"#)),
    }
}

pub async fn delete_channel(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let conn = &mut db::establish_connection();
    match ChannelRepository::delete(conn, id) {
        Ok(_) => {
            let _ = state.feed_tx.send(());
            Html(String::new()).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to delete channel {}: {}", id, e);
            Html(format!(r#"<p class="text-red-500 text-sm">Error: {e}</p>"#)).into_response()
        }
    }
}

pub async fn edit_channel_form(Path(id): Path<i32>) -> impl IntoResponse {
    let conn = &mut db::establish_connection();
    let channels = ChannelRepository::find_all(conn).unwrap_or_default();
    if let Some(channel) = channels.into_iter().find(|c| c.id == id) {
        let template = ChannelEditRowTemplate {
            channel,
            error: None,
        };
        render_template!(template)
    } else {
        Html("<p class='text-red-500'>Not found</p>".to_string())
    }
}

pub async fn channel_row(Path(id): Path<i32>) -> impl IntoResponse {
    let conn = &mut db::establish_connection();
    let channels = ChannelRepository::find_all(conn).unwrap_or_default();
    if let Some(channel) = channels.into_iter().find(|c| c.id == id) {
        let template = ChannelRowTemplate { channel };
        render_template!(template)
    } else {
        Html(String::new())
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
        Err(e) => return html_error(format!("Invalid CSV: {e}")),
    };

    let conn = &mut db::establish_connection();
    let (imported, skipped) = match insert_channels_into_db(conn, entries) {
        Ok(result) => result,
        Err(e) => return html_error(e),
    };

    if imported == 0 {
        return html_error(format!(
            "All {skipped} channel(s) are already in your list."
        ));
    }

    tokio::spawn(async move {
        if ChannelRepository::fetch_all_feeds().await {
            let _ = state.feed_tx.send(());
        }
    });

    htmx_redirect("/channels")
}

fn spawn_feed_refresh(state: AppState, channel_id: i32) {
    tokio::spawn(async move {
        info!("Spawning feed refresh for channel with ID {}", channel_id);
        if ChannelRepository::fetch_feed_for_channel(channel_id).await {
            let _ = state.feed_tx.send(());
        }
    });
}

async fn extract_csv_bytes(multipart: &mut Multipart) -> Result<Vec<u8>, Response> {
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name().unwrap_or("") == "csv_file" {
            return field
                .bytes()
                .await
                .map(|b| b.to_vec())
                .map_err(|e| html_error(format!("Failed to read file: {e}")));
        }
    }
    Err(html_error("No CSV file provided."))
}

/// Filters out already-existing channels, then bulk-inserts the rest in a
/// single transaction. Returns `(imported_count, skipped_count)` on success.
fn insert_channels_into_db(
    conn: &mut SqliteConnection,
    entries: Vec<NewChannel>,
) -> Result<(u32, u32), String> {
    let existing: HashSet<String> = ChannelRepository::find_all(conn)
        .unwrap_or_default()
        .into_iter()
        .map(|c| c.channel_id)
        .collect();

    let mut skipped = 0u32;
    let to_insert: Vec<NewChannel> = entries
        .into_iter()
        .filter(|e| {
            if existing.contains(&e.channel_id) {
                skipped += 1;
                false
            } else {
                true
            }
        })
        .collect();

    if to_insert.is_empty() {
        return Ok((0, skipped));
    }

    ChannelRepository::bulk_create(conn, &to_insert)
        .map(|imported| (imported, skipped))
        .map_err(|e| format!("Bulk insert failed: {e}"))
}

fn html_error(msg: impl std::fmt::Display) -> Response {
    Html(format!(r#"<p class="text-red-500 text-sm">{msg}</p>"#)).into_response()
}

fn htmx_redirect(path: &str) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert("HX-Redirect", path.parse().unwrap());
    (headers, Html(String::new())).into_response()
}

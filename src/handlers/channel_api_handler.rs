use askama::Template;
use axum::Form;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse};
use serde::Deserialize;
use tracing::info;
use crate::app_state::AppState;
use crate::db;
use crate::models::channel::NewChannel;
use crate::render_template;
use crate::repositories::channel_repository::ChannelRepository;
use crate::services::feed_reader_service::fetch_channel_name;
use crate::templates::{ChannelEditRowTemplate, ChannelRowTemplate};

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
        return Html(r#"<p class="text-red-500 text-sm">Channel ID is required.</p>"#.to_string()).into_response();
    }

    let channel_name = match fetch_channel_name(&channel_id).await {
        Ok(name) => name,
        Err(e) => return Html(format!(r#"<p class="text-red-500 text-sm">{e}</p>"#)).into_response(),
    };

    let conn = &mut db::establish_connection();
    match ChannelRepository::create(conn, NewChannel { channel_id, channel_name }) {
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
    let channel_name = form.channel_name.as_deref().unwrap_or("").trim().to_string();

    let validation_error = fetch_channel_name(&channel_id).await.err();

    let conn = &mut db::establish_connection();

    if let Some(ref e) = validation_error {
        if let Some(channel) = ChannelRepository::find_by_id(conn, id).unwrap_or_default() {
            let template = ChannelEditRowTemplate { channel, error: Some(e.clone()) };
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
        let template = ChannelEditRowTemplate { channel, error: None };
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

fn spawn_feed_refresh(state: AppState, channel_id: i32) {
    tokio::spawn(async move {
        info!("Spawning feed refresh for channel with ID {}", channel_id);
        if ChannelRepository::fetch_feed_for_channel(channel_id).await {
            let _ = state.feed_tx.send(());
        }
    });
}


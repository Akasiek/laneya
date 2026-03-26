use askama::Template;
use axum::response::IntoResponse;
use crate::db;
use crate::render_template;
use crate::repositories::channel_repository::ChannelRepository;
use crate::templates::ChannelsTemplate;

pub async fn channels_page() -> impl IntoResponse {
    let conn = &mut db::establish_connection();
    let channels = ChannelRepository::find_all(conn).unwrap_or_default();
    let template = ChannelsTemplate { channels };
    render_template!(template)
}


use askama::Template;
use axum::extract::Query;
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::db;
use crate::render_template;
use crate::repositories::video_repository::VideoRepository;
use crate::templates::VideoGridTemplate;

#[derive(Deserialize)]
pub struct VideoPageParams {
    pub page: Option<i64>,
}

pub async fn video_list(Query(params): Query<VideoPageParams>) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let conn = &mut db::establish_connection();
    let (videos, total_pages) =
        VideoRepository::find_paginated(conn, page).unwrap_or_else(|_| (vec![], 1));
    let template = VideoGridTemplate {
        videos,
        page,
        total_pages,
    };
    render_template!(template)
}

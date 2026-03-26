use crate::schema::videos;
use diesel::{Insertable, Queryable, Selectable};

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = videos)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Video {
    pub id: i32,
    pub video_id: String,
    pub channel_id: i32,
    pub title: String,
    pub video_url: String,
    pub thumbnail_url: String,
    pub published_at: String,
    pub updated_at: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = videos)]
pub struct NewVideo {
    pub video_id: String,
    pub channel_id: i32,
    pub title: String,
    pub video_url: String,
    pub thumbnail_url: String,
    pub published_at: String,
    pub updated_at: String,
}


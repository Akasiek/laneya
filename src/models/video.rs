use crate::config::Config;
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

#[derive(Queryable)]
pub struct VideoResource {
    pub id: i32,
    pub video_id: String,
    pub title: String,
    pub video_url: String,
    pub thumbnail_url: String,
    pub published_at: String,
    pub published_at_relative: String,
    pub channel_id: String,
    pub channel_name: String,
}

impl VideoResource {
    pub fn playback_url(&self) -> String {
        let Some(base_url) = &Config::get().youtube_frontend_url else {
            return self.video_url.clone();
        };

        let mut url = base_url.clone();
        let path = format!("{}/watch", url.path().trim_end_matches('/'));
        url.set_path(&path);
        url.set_query(None);
        url.set_fragment(None);
        url.query_pairs_mut().append_pair("v", &self.video_id);
        url.to_string()
    }
}

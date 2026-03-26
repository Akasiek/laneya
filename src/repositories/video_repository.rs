use diesel::prelude::*;
use crate::models::video::{NewVideo, Video};
use crate::schema::videos;
use crate::services::feed_reader_service::{FeedEntry, YoutubeFeed};

pub struct VideoRepository;

impl VideoRepository {
    /// Upsert feed entries for a given channel.
    /// Returns true if any new videos were inserted or existing ones changed.
    pub fn upsert_from_feed(
        conn: &mut SqliteConnection,
        channel_db_id: i32,
        feed: &YoutubeFeed,
    ) -> anyhow::Result<bool> {
        let filter_shorts = crate::config::Config::get().filter_shorts;
        let mut changed = false;

        for entry in &feed.entries {
            if filter_shorts && entry.link.href.starts_with("https://www.youtube.com/shorts/") {
                continue;
            }
            let existing: Option<(String, String)> = videos::table
                .filter(videos::video_id.eq(&entry.video_id))
                .select((videos::title, videos::updated_at))
                .first(conn)
                .optional()?;

            let is_new_or_changed = match existing {
                None => true,
                Some((title, updated_at)) => {
                    title != entry.title || updated_at != entry.updated
                }
            };

            if is_new_or_changed {
                let new_video = Self::new_video_from_entry(entry, channel_db_id);

                diesel::insert_into(videos::table)
                    .values(&new_video)
                    .on_conflict(videos::video_id)
                    .do_update()
                    .set((
                        videos::title.eq(&new_video.title),
                        videos::video_url.eq(&new_video.video_url),
                        videos::thumbnail_url.eq(&new_video.thumbnail_url),
                        videos::published_at.eq(&new_video.published_at),
                        videos::updated_at.eq(&new_video.updated_at),
                    ))
                    .execute(conn)?;

                changed = true;
            }
        }

        Ok(changed)
    }

    fn new_video_from_entry(entry: &FeedEntry, channel_db_id: i32) -> NewVideo {
        NewVideo {
            video_id: entry.video_id.clone(),
            channel_id: channel_db_id,
            title: entry.title.clone(),
            video_url: entry.link.href.clone(),
            thumbnail_url: entry.media_group.thumbnail.url.clone(),
            published_at: entry.published.clone(),
            updated_at: entry.updated.clone(),
        }
    }

    pub fn find_all_recent(conn: &mut SqliteConnection) -> anyhow::Result<Vec<Video>> {
        let result = videos::table
            .order(videos::published_at.desc())
            .load::<Video>(conn)?;
        Ok(result)
    }

    pub fn delete_by_channel_id(conn: &mut SqliteConnection, channel_db_id: i32) -> anyhow::Result<()> {
        diesel::delete(videos::table.filter(videos::channel_id.eq(channel_db_id)))
            .execute(conn)?;
        Ok(())
    }
}

use crate::models::video::{NewVideo, VideoResource};
use crate::schema::{channels, videos};
use crate::services::feed_reader_service::{FeedEntry, YoutubeFeed};
use diesel::prelude::*;

pub struct VideoRepository;

impl VideoRepository {
    /// Upsert feed entries for a given channel.
    /// Returns true if any new videos were inserted or existing ones changed.
    pub fn upsert_from_feed(
        conn: &mut SqliteConnection,
        channel_db_id: i32,
        feed: &YoutubeFeed,
    ) -> anyhow::Result<bool> {
        let filter_out_shorts = crate::config::Config::get().filter_out_shorts;
        let mut changed = false;

        for entry in &feed.entries {
            if filter_out_shorts
                && entry
                    .link
                    .href
                    .starts_with("https://www.youtube.com/shorts/")
            {
                continue;
            }
            let existing: Option<(String, String)> = videos::table
                .filter(videos::video_id.eq(&entry.video_id))
                .select((videos::title, videos::updated_at))
                .first(conn)
                .optional()?;

            let is_new_or_changed = match existing {
                None => true,
                Some((title, updated_at)) => title != entry.title || updated_at != entry.updated,
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

    pub fn find_paginated(
        conn: &mut SqliteConnection,
        page: i64,
    ) -> anyhow::Result<(Vec<VideoResource>, i64)> {
        let per_page = crate::config::Config::get().per_page;

        let total: i64 = videos::table.count().get_result(conn)?;
        let total_pages = ((total + per_page - 1) / per_page).max(1);
        let page = page.clamp(1, total_pages);
        let offset = (page - 1) * per_page;

        let rows = videos::table
            .inner_join(channels::table)
            .select((
                videos::id,
                videos::video_id,
                videos::title,
                videos::video_url,
                videos::thumbnail_url,
                videos::published_at,
                channels::channel_id,
                channels::channel_name,
            ))
            .order(videos::published_at.desc())
            .limit(per_page)
            .offset(offset)
            .load::<VideoResource>(conn)
            .map(|rows| {
                rows.into_iter()
                    .map(|mut v| {
                        v.published_at = format_published_at(&v.published_at);
                        v
                    })
                    .collect()
            })?;

        Ok((rows, total_pages))
    }

    pub fn delete_by_channel_id(
        conn: &mut SqliteConnection,
        channel_db_id: i32,
    ) -> anyhow::Result<()> {
        diesel::delete(videos::table.filter(videos::channel_id.eq(channel_db_id))).execute(conn)?;
        Ok(())
    }
}

fn format_published_at(iso: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(iso)
        .map(|dt| {
            dt.with_timezone(&chrono::Local)
                .format("%d %b %Y %H:%M")
                .to_string()
        })
        .unwrap_or_else(|_| iso.to_string())
}

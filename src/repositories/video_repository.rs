use crate::config::Config;
use crate::models::video::{NewVideo, VideoResource};
use crate::schema::{channels, videos};
use crate::services::feed_reader_service::{FeedEntry, YoutubeFeed};
use diesel::prelude::*;
use tracing::error;

pub struct VideoRepository;

const SHORT_URL: &'static str = "https://www.youtube.com/shorts/";

impl VideoRepository {
    /// Upsert feed entries for a given channel.
    /// Returns true if any new videos were inserted or existing ones changed.
    pub fn upsert_from_feed(
        conn: &mut SqliteConnection,
        channel_id: i32,
        feed: &YoutubeFeed,
    ) -> anyhow::Result<bool> {
        let mut changed = false;

        for video in &feed.entries {
            match Self::upsert_video(conn, video, channel_id) {
                Ok(video_changed) => {
                    if video_changed {
                        changed = true;
                    }
                }
                Err(_) => (),
            }
        }

        Ok(changed)
    }

    /// Upsert a single video entry.
    /// Returns Ok(true) if the video was new or updated, Ok(false) if it was unchanged, and Err(()) on failure.
    fn upsert_video(
        conn: &mut SqliteConnection,
        video: &FeedEntry,
        channel_id: i32,
    ) -> Result<bool, ()> {
        if Self::should_filter_video(video) {
            return Ok(false);
        }

        let existing: Option<(String, String)> = match videos::table
            .filter(videos::video_id.eq(&video.video_id))
            .select((videos::title, videos::updated_at))
            .first(conn)
            .optional()
        {
            Ok(e) => e,
            Err(e) => {
                error!("Failed to query existing video {}: {}", video.video_id, e);
                return Err(());
            }
        };

        let is_new_or_changed = match existing {
            None => true,
            Some((title, updated_at)) => title != video.title || updated_at != video.updated,
        };

        if is_new_or_changed {
            let new_video = Self::new_video_from_entry(video, channel_id);

            match diesel::insert_into(videos::table)
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
                .execute(conn)
            {
                Ok(_) => (),
                Err(e) => {
                    error!("Failed to upsert video {}: {}", video.video_id, e);
                    return Err(());
                }
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn should_filter_video(entry: &FeedEntry) -> bool {
        let filter_out_shorts = Config::get().filter_out_shorts;
        let is_short = entry.link.href.starts_with(SHORT_URL);

        filter_out_shorts && is_short
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
        let per_page = Config::get().videos_per_page;

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
        channel_id: i32,
    ) -> anyhow::Result<()> {
        diesel::delete(videos::table.filter(videos::channel_id.eq(channel_id))).execute(conn)?;
        Ok(())
    }
}

fn format_published_at(timestamp: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(timestamp)
        .map(|dt| {
            dt.with_timezone(&chrono::Local)
                .format("%d %b %Y %H:%M")
                .to_string()
        })
        .unwrap_or_else(|_| timestamp.to_string())
}

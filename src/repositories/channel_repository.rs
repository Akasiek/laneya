use crate::app_state::AppState;
use crate::db;
use crate::models::channel::{Channel, NewChannel};
use crate::repositories::video_repository::VideoRepository;
use crate::services::feed_reader_service::{read_channel_feed, read_channels_feed};
use crate::services::ws_service;
use diesel::prelude::*;
use std::collections::HashSet;
use tracing::info;

pub struct ChannelRepository;

impl ChannelRepository {
    pub fn find_all(conn: &mut SqliteConnection) -> anyhow::Result<Vec<Channel>> {
        use crate::schema::channels::dsl::*;
        let result = channels.order(channel_name.asc()).load::<Channel>(conn)?;
        Ok(result)
    }

    pub fn create(conn: &mut SqliteConnection, new_channel: NewChannel) -> anyhow::Result<Channel> {
        use crate::schema::channels::dsl::*;
        let channel = diesel::insert_into(channels)
            .values(&new_channel)
            .get_result::<Channel>(conn)?;

        Ok(channel)
    }

    /// Attempts to bulk insert channels, skipping any that already exist based on `channel_id`.
    /// Returns a tuple of (number_inserted, number_skipped).
    pub fn bulk_create(
        conn: &mut SqliteConnection,
        new_channels: Vec<NewChannel>,
    ) -> anyhow::Result<(u32, u32)> {
        use crate::schema::channels::dsl::*;

        let existing: HashSet<String> = Self::find_all(conn)
            .unwrap_or_default()
            .into_iter()
            .map(|c| c.channel_id)
            .collect();

        let mut skipped: u32 = 0;
        let to_insert: Vec<NewChannel> = new_channels
            .into_iter()
            .filter(|ch| {
                if existing.contains(&ch.channel_id) {
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

        conn.transaction(|conn| {
            to_insert
                .iter()
                .map(|ch| diesel::insert_into(channels).values(ch).execute(conn))
                .collect::<QueryResult<Vec<_>>>()
        })
        .map(|v| (v.len() as u32, skipped))
        .map_err(anyhow::Error::from)
    }

    pub fn update(
        conn: &mut SqliteConnection,
        channel_id_pk: i32,
        name: String,
        ch_id: String,
    ) -> anyhow::Result<()> {
        use crate::schema::channels::dsl::*;
        diesel::update(channels.filter(id.eq(channel_id_pk)))
            .set((channel_name.eq(name), channel_id.eq(ch_id)))
            .execute(conn)?;

        Ok(())
    }

    pub fn delete(conn: &mut SqliteConnection, channel_id_pk: i32) -> anyhow::Result<()> {
        use crate::schema::channels::dsl::*;
        VideoRepository::delete_by_channel_id(conn, channel_id_pk)?;
        diesel::delete(channels.filter(id.eq(channel_id_pk))).execute(conn)?;

        Ok(())
    }

    pub fn find_by_id(
        conn: &mut SqliteConnection,
        channel_id_pk: i32,
    ) -> anyhow::Result<Option<Channel>> {
        use crate::schema::channels::dsl::*;
        let result = channels
            .filter(id.eq(channel_id_pk))
            .first::<Channel>(conn)
            .optional()?;
        Ok(result)
    }

    pub fn spawn_channel_feed_refresh(state: AppState, channel_id: i32) {
        tokio::spawn(async move {
            info!("Starting feed refresh for channel {}", channel_id);
            if ChannelRepository::fetch_feed_for_channel(channel_id).await {
                ws_service::send_refresh_notification(state).await;
            }
        });
    }

    pub async fn fetch_feed_for_channel(channel_id_pk: i32) -> bool {
        let conn = &mut db::establish_connection();

        let channel = match Self::find_by_id(conn, channel_id_pk) {
            Ok(Some(c)) => c,
            Ok(None) => return false,
            Err(e) => {
                tracing::error!("Failed to fetch channel {}: {}", channel_id_pk, e);
                return false;
            }
        };

        let client = reqwest::Client::new();
        let feed = match read_channel_feed(&channel, &client).await {
            Ok(feed) => feed,
            Err(e) => {
                tracing::error!("Failed to fetch feed for channel {}: {}", channel.id, e);
                return false;
            }
        };

        match VideoRepository::upsert_from_feed(conn, channel.id, &feed) {
            Ok(changed) => changed,
            Err(e) => {
                tracing::error!("Failed to upsert feed for channel {}: {}", channel.id, e);
                false
            }
        }
    }

    pub async fn spawn_all_feeds_refresh(state: AppState) {
        tokio::spawn(async move {
            info!("Starting full feed refresh for all channels");
            if ChannelRepository::fetch_all_feeds().await {
                ws_service::send_refresh_notification(state).await;
            }
        });
    }

    pub async fn fetch_all_feeds() -> bool {
        let conn = &mut db::establish_connection();

        let channels_list = Self::find_all(conn).unwrap_or_else(|e| {
            tracing::error!("Failed to fetch channels: {}", e);
            Vec::new()
        });

        let client = std::sync::Arc::new(reqwest::Client::new());
        let results = read_channels_feed(channels_list, client).await;

        let mut any_changed = false;
        for (channel, feed) in results {
            let Some(feed) = feed else { continue };

            match VideoRepository::upsert_from_feed(conn, channel.id, &feed) {
                Err(e) => tracing::error!(
                    "Failed to upsert videos for channel {} ({}): {}",
                    channel.channel_name,
                    channel.id,
                    e
                ),
                Ok(false) => {}
                Ok(true) => {
                    info!(
                        "New/updated videos for channel {} ({})",
                        channel.channel_name, channel.id
                    );
                    any_changed = true;
                }
            }
        }

        any_changed
    }
}

use crate::db;
use crate::models::channel::{Channel, NewChannel};
use crate::repositories::video_repository::VideoRepository;
use crate::services::feed_reader_service::{read_channel_feed, read_channels_feed};
use diesel::prelude::*;

pub struct ChannelRepository;

impl ChannelRepository {
    pub fn find_all(conn: &mut SqliteConnection) -> anyhow::Result<Vec<Channel>> {
        use crate::schema::channels::dsl::*;
        let result = channels.order(channel_name.asc()).load::<Channel>(conn)?;
        Ok(result)
    }

    pub fn create(conn: &mut SqliteConnection, new_channel: NewChannel) -> anyhow::Result<i32> {
        use crate::schema::channels::dsl::*;
        let new_id = diesel::insert_into(channels)
            .values(&new_channel)
            .returning(id)
            .get_result::<i32>(conn)?;

        Ok(new_id)
    }

    pub fn bulk_create(
        conn: &mut SqliteConnection,
        new_channels: &[NewChannel],
    ) -> anyhow::Result<u32> {
        use crate::schema::channels::dsl::*;
        conn.transaction(|conn| {
            new_channels
                .iter()
                .map(|ch| diesel::insert_into(channels).values(ch).execute(conn))
                .collect::<QueryResult<Vec<_>>>()
        })
        .map(|v| v.len() as u32)
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
                    tracing::info!(
                        "New/updated videos for channel {} ({})",
                        channel.channel_name,
                        channel.id
                    );
                    any_changed = true;
                }
            }
        }

        any_changed
    }
}

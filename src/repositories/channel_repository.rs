use crate::models::channel::Channel;
use crate::repositories::video_repository::VideoRepository;
use crate::services::feed_reader_service::read_channels_feed;
use diesel::{RunQueryDsl, SqliteConnection};
use crate::db;

pub struct ChannelRepository;

impl ChannelRepository {
    pub fn find_all(conn: &mut SqliteConnection) -> anyhow::Result<Vec<Channel>> {
        use crate::schema::channels::dsl::*;
        let result = channels.load::<Channel>(conn)?;
        Ok(result)
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
            if let Some(feed) = feed {
                match VideoRepository::upsert_from_feed(conn, channel.id, &feed) {
                    Ok(changed) => {
                        if changed {
                            tracing::info!("New/updated videos for channel {} ({})", channel.channel_name, channel.id);
                            any_changed = true;
                        }
                    }
                    Err(e) => tracing::error!(
                        "Failed to upsert videos for channel {} ({}): {}",
                        channel.channel_name, channel.id, e
                    ),
                }
            }
        }
        any_changed
    }
}

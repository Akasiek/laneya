use crate::repositories::channel_repository::ChannelRepository;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time;

pub fn start_feed_refresh_job(feed_tx: broadcast::Sender<()>) {
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(5 * 60));
        loop {
            interval.tick().await;
            tracing::info!("Starting scheduled feed refresh...");
            let changed = ChannelRepository::fetch_all_feeds().await;

            if changed {
                tracing::info!("Feed changed, notifying clients.");
                let _ = feed_tx.send(());
            } else {
                tracing::info!("No changes detected in feeds.");
            }
        }
    });
}

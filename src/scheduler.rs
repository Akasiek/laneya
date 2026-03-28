use crate::config::Config;
use crate::repositories::channel_repository::ChannelRepository;
use std::time::Duration;
use tokio::sync::broadcast::Sender;
use tokio::time;
use crate::services::ws_service;

pub fn spawn_feed_refresh_job(feed_tx: Sender<()>) {
    tokio::spawn(async move {
        feed_refresh_job(feed_tx).await;
    });
}

async fn feed_refresh_job(feed_tx: Sender<()>) {
    let interval_mins = Config::get().feed_refresh_interval_mins;
    let mut interval = time::interval(Duration::from_secs(interval_mins * 60));
    tracing::info!("Feed refresh interval set to {} minutes.", interval_mins);

    loop {
        interval.tick().await;
        refresh_feed(feed_tx.clone()).await;
    }
}

pub async fn refresh_feed(feed_tx: Sender<()>) {
    tracing::info!("Starting scheduled feed refresh...");
    let changed = ChannelRepository::fetch_all_feeds().await;

    if changed {
        tracing::info!("Feed changed, notifying clients.");
        ws_service::send_refresh_notification(feed_tx).await;
    } else {
        tracing::info!("No changes detected in feeds.");
    }
}
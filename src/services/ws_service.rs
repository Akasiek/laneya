use tokio::sync::broadcast;

pub async fn send_refresh_notification(broadcast_channel: broadcast::Sender<()>) {
    let _ = broadcast_channel.send(());
}
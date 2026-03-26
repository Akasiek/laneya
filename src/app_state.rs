use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    /// Sender for notifying WebSocket clients about feed updates.
    pub feed_tx: broadcast::Sender<()>,
}

impl AppState {
    pub fn new() -> Self {
        let (feed_tx, _) = broadcast::channel(16);
        Self { feed_tx }
    }
}


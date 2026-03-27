use crate::app_state::AppState;

pub async fn send_refresh_notification(state: AppState) {
    let _ = state.feed_tx.send(());
}
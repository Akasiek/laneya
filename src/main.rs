
use laneya::{db, scheduler, tracer, web_server};
use laneya::app_state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracer::init();
    db::init();

    let state = AppState::new();
    scheduler::start_feed_refresh_job(state.feed_tx.clone());
    web_server::init(state).await;
}

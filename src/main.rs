use laneya::{scheduler, tracer, web_server};
use laneya::app_state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracer::init_tracer();

    let state = AppState::new();
    scheduler::start_feed_refresh_job(state.feed_tx.clone());
    web_server::init_web_server(state).await;
}

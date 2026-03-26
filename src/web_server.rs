use crate::app_state::AppState;
use crate::router::get_app_router;
use listenfd::ListenFd;
use tokio::net::TcpListener;
use tracing::info;

pub async fn init(state: AppState) {
    let router = get_app_router(state);
    let listener = get_app_listener().await;

    info!(
        "Server listening on http://{}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, router).await.unwrap();
}


pub async fn get_app_listener() -> TcpListener {
    let mut listenfd = ListenFd::from_env();
    match listenfd.take_tcp_listener(0).unwrap() {
        Some(listener) => {
            info!("Using listener from environment (e.g., systemd socket activation)");
            listener.set_nonblocking(true).unwrap();
            TcpListener::from_std(listener).unwrap()
        }
        // otherwise fall back to local listening
        None => TcpListener::bind("127.0.0.1:8080").await.unwrap(),
    }
}

use axum::{
    extract::{Request, State},
    routing::{get, post},
    Json, Router,
};
use tracing::info;

#[derive(Clone, Debug)]
struct AppState {}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let router = Router::new()
        .route("/compile", post(compile))
        .with_state(AppState {});
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
async fn compile(State(state): State<AppState>, body: String) -> &'static str {
    info!("state: {:?}", state);
    "compile"
}

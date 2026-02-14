mod app;
mod config;
mod error;
mod routes;
mod s3;

use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = config::AppConfig::load().expect("failed to load configuration");
    let listen = config.listen.clone();
    let router = app::build_router(&config);

    let listener = tokio::net::TcpListener::bind(&listen)
        .await
        .expect("failed to bind");
    tracing::info!("listening on {listen}");

    axum::serve(listener, router).await.expect("server error");
}

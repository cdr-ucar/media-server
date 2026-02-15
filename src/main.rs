mod app;
mod config;
mod error;
mod routes;
mod s3;

use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    ctrlc::set_handler(|| {
        tracing::info!("Received termination signal, exiting...");
        std::process::exit(0);
    })?;

    let config = config::AppConfig::load()?;
    let listen = config.listen.clone();
    let router = app::build_router(&config);

    let listener = tokio::net::TcpListener::bind(&listen).await?;
    tracing::info!("Listening on {listen}");

    axum::serve(listener, router).await?;

    Ok(())
}

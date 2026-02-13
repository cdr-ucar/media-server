use axum::Router;
use axum::routing::get;

use crate::config::AppConfig;
use crate::s3::new_file_server;

pub fn build_router(config: &AppConfig) -> Router {
    let server = new_file_server(config);

    Router::new()
        .route("/{config_name}/{*file_path}", get(crate::routes::get_file))
        .with_state(server)
}

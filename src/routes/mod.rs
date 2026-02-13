use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};

use crate::error::AppError;
use crate::s3::{FileResponse, FileServer};

pub async fn get_file(
    State(server): State<Arc<dyn FileServer>>,
    Path((config_name, file_path)): Path<(String, String)>,
) -> Result<Response, AppError> {
    let response = server.get_file(&config_name, &file_path).await?;

    match response {
        FileResponse::Redirect(url) => {
            let mut resp = StatusCode::FOUND.into_response();
            resp.headers_mut().insert(
                "Location",
                HeaderValue::from_str(&url).map_err(|e| AppError::S3Error(e.to_string()))?,
            );
            Ok(resp)
        }

        FileResponse::Stream { content_type, body } => {
            let mut resp = Response::new(body);
            resp.headers_mut().insert(
                "Content-Type",
                HeaderValue::from_str(&content_type)
                    .map_err(|e| AppError::S3Error(e.to_string()))?,
            );
            Ok(resp)
        }
    }
}

#[cfg(test)]
mod tests;

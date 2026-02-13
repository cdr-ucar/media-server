use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub enum AppError {
    ConfigNotFound(String),
    ObjectNotFound(String),
    S3Error(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConfigNotFound(name) => write!(f, "config not found: {name}"),
            Self::ObjectNotFound(key) => write!(f, "object not found: {key}"),
            Self::S3Error(msg) => write!(f, "S3 error: {msg}"),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Self::ConfigNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            Self::ObjectNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            Self::S3Error(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        tracing::error!("{message}");
        (status, message).into_response()
    }
}

#[cfg(test)]
mod tests;

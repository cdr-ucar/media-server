use super::*;
use axum::Router;
use axum::body::Body;
use axum::routing::get;
use http_body_util::BodyExt;
use std::future::Future;
use std::pin::Pin;
use tower::ServiceExt;

struct MockFileServer {
    response: std::sync::Mutex<Option<Result<FileResponse, AppError>>>,
}

impl FileServer for MockFileServer {
    fn get_file(
        &self,
        config_name: &str,
        _file_path: &str,
    ) -> Pin<Box<dyn Future<Output = Result<FileResponse, AppError>> + Send + '_>> {
        let config_name = config_name.to_string();
        let result = self.response.lock().unwrap().take();
        Box::pin(async move { result.unwrap_or(Err(AppError::ConfigNotFound(config_name))) })
    }
}

fn test_router(mock: MockFileServer) -> Router {
    let server: Arc<dyn FileServer> = Arc::new(mock);
    Router::new()
        .route("/{config_name}/{*file_path}", get(get_file))
        .with_state(server)
}

fn request(uri: &str) -> axum::http::Request<Body> {
    axum::http::Request::builder()
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn redirect_mode_returns_302() {
    let mock = MockFileServer {
        response: std::sync::Mutex::new(Some(Ok(FileResponse::Redirect(
            "https://s3.example.com/presigned".into(),
        )))),
    };
    let app = test_router(mock);
    let resp = app.oneshot(request("/photos/img.jpg")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FOUND);
    assert_eq!(
        resp.headers().get("location").unwrap(),
        "https://s3.example.com/presigned"
    );
}

#[tokio::test]
async fn proxy_mode_streams_body() {
    let mock = MockFileServer {
        response: std::sync::Mutex::new(Some(Ok(FileResponse::Stream {
            content_type: "image/jpeg".into(),
            body: Body::from("fake-image-data"),
        }))),
    };
    let app = test_router(mock);
    let resp = app.oneshot(request("/docs/file.pdf")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.headers().get("content-type").unwrap(), "image/jpeg");
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"fake-image-data");
}

#[tokio::test]
async fn unknown_config_returns_404() {
    let mock = MockFileServer {
        response: std::sync::Mutex::new(Some(Err(AppError::ConfigNotFound("nope".into())))),
    };
    let app = test_router(mock);
    let resp = app.oneshot(request("/nope/file.txt")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn missing_file_returns_404() {
    let mock = MockFileServer {
        response: std::sync::Mutex::new(Some(Err(AppError::ObjectNotFound("missing.txt".into())))),
    };
    let app = test_router(mock);
    let resp = app.oneshot(request("/photos/missing.txt")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn s3_error_returns_500() {
    let mock = MockFileServer {
        response: std::sync::Mutex::new(Some(Err(AppError::S3Error("connection refused".into())))),
    };
    let app = test_router(mock);
    let resp = app.oneshot(request("/photos/img.jpg")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

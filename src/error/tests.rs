use super::*;

#[test]
fn config_not_found_is_404() {
    let resp = AppError::ConfigNotFound("x".into()).into_response();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[test]
fn object_not_found_is_404() {
    let resp = AppError::ObjectNotFound("key".into()).into_response();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[test]
fn s3_error_is_500() {
    let resp = AppError::S3Error("boom".into()).into_response();
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

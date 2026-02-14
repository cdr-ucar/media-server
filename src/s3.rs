use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use aws_credential_types::Credentials;
use aws_sdk_s3::config::Region;
use aws_sdk_s3::presigning::PresigningConfig;
use axum::body::Body;

use crate::config::{AppConfig, BucketConfig};
use crate::error::AppError;

struct BucketClient {
    client: aws_sdk_s3::Client,
    bucket_name: String,
    proxy: bool,
    presign_expiry: Duration,
}

pub struct S3Clients {
    buckets: HashMap<String, BucketClient>,
}

impl S3Clients {
    pub fn from_config(config: &AppConfig) -> Self {
        let mut buckets = HashMap::new();

        for (name, bc) in &config.buckets {
            let client = build_s3_client(bc);
            let presign_expiry =
                Duration::from_secs(bc.presign_expiry_secs.unwrap_or(config.presign_expiry_secs));

            buckets.insert(
                name.clone(),
                BucketClient {
                    client,
                    bucket_name: bc.bucket_name.clone(),
                    proxy: bc.proxy,
                    presign_expiry,
                },
            );
        }

        Self { buckets }
    }
}

fn build_s3_client(bc: &BucketConfig) -> aws_sdk_s3::Client {
    let credentials = Credentials::new(
        bc.access_key.clone(),
        bc.secret_key.clone(),
        None,
        None,
        "celia-media",
    );

    let s3_config = aws_sdk_s3::Config::builder()
        .endpoint_url(&bc.endpoint_url)
        .region(Region::new(bc.region.clone()))
        .credentials_provider(credentials)
        .force_path_style(bc.force_path_style)
        .behavior_version_latest()
        .build();

    aws_sdk_s3::Client::from_conf(s3_config)
}

impl S3Clients {
    async fn redirect_file(
        &self,
        bc: &BucketClient,
        file_path: &str,
    ) -> Result<FileResponse, AppError> {
        // head_object to verify existence and distinguish 404 from other errors
        let head_result = bc
            .client
            .head_object()
            .bucket(&bc.bucket_name)
            .key(file_path)
            .send()
            .await;

        match head_result {
            Ok(_) => {}
            Err(err) => {
                let service_err = err.into_service_error();
                if service_err.is_not_found() {
                    return Err(AppError::ObjectNotFound(file_path.to_string()));
                }
                return Err(AppError::S3Error(service_err.to_string()));
            }
        }

        let presign_config = PresigningConfig::builder()
            .expires_in(bc.presign_expiry)
            .build()
            .map_err(|e| AppError::S3Error(e.to_string()))?;

        let presigned = bc
            .client
            .get_object()
            .bucket(&bc.bucket_name)
            .key(file_path)
            .presigned(presign_config)
            .await
            .map_err(|e| AppError::S3Error(e.to_string()))?;

        Ok(FileResponse::Redirect(presigned.uri().to_string()))
    }

    async fn proxy_file(
        &self,
        bc: &BucketClient,
        file_path: &str,
    ) -> Result<FileResponse, AppError> {
        let result = bc
            .client
            .get_object()
            .bucket(&bc.bucket_name)
            .key(file_path)
            .send()
            .await;

        match result {
            Ok(output) => {
                let content_type = output
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string();
                let reader = output.body.into_async_read();
                let stream = tokio_util::io::ReaderStream::new(reader);
                let body = Body::from_stream(stream);
                Ok(FileResponse::Stream { content_type, body })
            }
            Err(err) => {
                let service_err = err.into_service_error();
                if service_err.is_no_such_key() {
                    return Err(AppError::ObjectNotFound(file_path.to_string()));
                }
                Err(AppError::S3Error(service_err.to_string()))
            }
        }
    }
}

pub enum FileResponse {
    Redirect(String),
    Stream { content_type: String, body: Body },
}

pub trait FileServer: Send + Sync {
    fn get_file(
        &self,
        config_name: &str,
        file_path: &str,
    ) -> Pin<Box<dyn Future<Output = Result<FileResponse, AppError>> + Send + '_>>;
}

impl FileServer for S3Clients {
    fn get_file(
        &self,
        config_name: &str,
        file_path: &str,
    ) -> Pin<Box<dyn Future<Output = Result<FileResponse, AppError>> + Send + '_>> {
        let config_name = config_name.to_string();
        let file_path = file_path.to_string();
        Box::pin(async move {
            let bc = self
                .buckets
                .get(&config_name)
                .ok_or_else(|| AppError::ConfigNotFound(config_name.clone()))?;

            if bc.proxy {
                self.proxy_file(bc, &file_path).await
            } else {
                self.redirect_file(bc, &file_path).await
            }
        })
    }
}

pub fn new_file_server(config: &AppConfig) -> Arc<dyn FileServer> {
    Arc::new(S3Clients::from_config(config))
}

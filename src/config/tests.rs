use super::*;

#[test]
fn parse_valid_config() {
    let yaml = r#"
listen: "127.0.0.1:3000"
presign_expiry_secs: 600
buckets:
  photos:
    endpoint_url: "https://minio.example.com"
    bucket_name: "my-photos"
    access_key: "AKIA"
    secret_key: "secret"
    region: "eu-west-1"
    force_path_style: false
    presign_expiry_secs: 900
  docs:
    endpoint_url: "http://minio.internal:9000"
    bucket_name: "docs"
    access_key: "AKIA2"
    secret_key: "secret2"
    proxy: true
"#;
    let config: AppConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.listen, "127.0.0.1:3000");
    assert_eq!(config.presign_expiry_secs, 600);
    assert_eq!(config.buckets.len(), 2);

    let photos = &config.buckets["photos"];
    assert_eq!(photos.endpoint_url, "https://minio.example.com");
    assert_eq!(photos.region, "eu-west-1");
    assert!(!photos.force_path_style);
    assert_eq!(photos.presign_expiry_secs, Some(900));
    assert!(!photos.proxy);

    let docs = &config.buckets["docs"];
    assert!(docs.proxy);
    assert_eq!(docs.region, "us-east-1");
    assert!(docs.force_path_style);
    assert_eq!(docs.presign_expiry_secs, None);
}

#[test]
fn defaults_applied() {
    let yaml = r#"
buckets:
  test:
    endpoint_url: "http://localhost:9000"
    bucket_name: "test"
    access_key: "key"
    secret_key: "secret"
"#;
    let config: AppConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.listen, "[::]:8080");
    assert_eq!(config.presign_expiry_secs, 300);

    let bucket = &config.buckets["test"];
    assert_eq!(bucket.region, "us-east-1");
    assert!(bucket.force_path_style);
    assert!(!bucket.proxy);
    assert_eq!(bucket.presign_expiry_secs, None);
}

#[test]
fn missing_required_field_errors() {
    let yaml = r#"
buckets:
  bad:
    endpoint_url: "http://localhost:9000"
    bucket_name: "test"
    access_key: "key"
"#;
    let result: Result<AppConfig, _> = serde_yaml::from_str(yaml);
    assert!(result.is_err());
}

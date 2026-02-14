use std::{
    io::Write,
    time::{SystemTime, UNIX_EPOCH},
};

use super::*;

fn unique_tmp_path(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!(
        "credconfig_test_{name}_{nanos}_{}",
        std::process::id()
    ));
    path
}

#[test]
fn test_read_plain_credential() {
    let credential = CredentialConfig::Plain {
        plain: "aaaaaa".to_string(),
    };
    let secret: String = credential.into();
    assert_eq!(secret, "aaaaaa");
}

#[test]
fn test_read_credential_from_file() {
    let path = unique_tmp_path("path_reads_file_contents");

    {
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"secret\n").unwrap();
    }

    let credential = CredentialConfig::Path {
        path: path.to_string_lossy().to_string(),
    };

    let secret: String = credential.into();
    assert_eq!(secret, "secret\n");

    let _ = std::fs::remove_file(&path);
}

#[test]
#[should_panic]
fn test_read_credential_from_missing_file_panics() {
    let path = unique_tmp_path("path_missing");
    let path = path.to_string_lossy().to_string();

    let credential = CredentialConfig::Path { path };

    let _: String = credential.into();
}

#[test]
fn test_read_credential_from_env_var() {
    // Use a unique key to avoid collisions with parallel tests.
    let key = format!(
        "CREDTEST_ENV_OK_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );

    unsafe { std::env::set_var(&key, "from_env") };

    let credential = CredentialConfig::Env { env: key.clone() };
    let secret: String = credential.into();
    assert_eq!(secret, "from_env");

    unsafe { std::env::remove_var(&key) };
}

#[test]
#[should_panic]
fn into_string_env_panics_if_missing() {
    let key = format!(
        "CREDTEST_ENV_MISSING_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );

    unsafe { std::env::remove_var(&key) };

    let credential = CredentialConfig::Env { env: key.clone() };

    let _: String = credential.into();
}

#[test]
fn parse_valid_config() {
    let yaml = r#"
listen: "127.0.0.1:3000"
presign_expiry_secs: 600
buckets:
  photos:
    endpoint_url: "https://minio.example.com"
    bucket_name: "my-photos"
    access_key:
        env: "AKIA"
    secret_key:
        plain: "secret"
    region: "eu-west-1"
    force_path_style: false
    presign_expiry_secs: 900
  docs:
    endpoint_url: "http://minio.internal:9000"
    bucket_name: "docs"
    access_key:
        env: "AKIA2"
    secret_key:
        path: "/secret2"
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
    assert_eq!(
        photos.access_key,
        CredentialConfig::Env { env: "AKIA".into() }
    );
    assert_eq!(
        photos.secret_key,
        CredentialConfig::Plain {
            plain: "secret".into()
        }
    );

    let docs = &config.buckets["docs"];
    assert!(docs.proxy);
    assert_eq!(docs.region, "us-east-1");
    assert!(docs.force_path_style);
    assert_eq!(docs.presign_expiry_secs, None);
    assert_eq!(
        docs.access_key,
        CredentialConfig::Env {
            env: "AKIA2".into()
        }
    );
    assert_eq!(
        docs.secret_key,
        CredentialConfig::Path {
            path: "/secret2".into()
        }
    );
}

#[test]
fn defaults_applied() {
    let yaml = r#"
buckets:
  test:
    endpoint_url: "http://localhost:9000"
    bucket_name: "test"
    access_key:
        plain: "key"
    secret_key:
        plain: "secret"
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
    bucket_name:
        plain: "test"
    access_key:
        plain: "key"
"#;
    let result: Result<AppConfig, _> = serde_yaml::from_str(yaml);
    assert!(result.is_err());
}

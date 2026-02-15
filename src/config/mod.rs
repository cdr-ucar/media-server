use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

mod constants;

fn default_region() -> String {
    constants::DEFAULT_S3_REGION.to_string()
}

fn default_force_path_style() -> bool {
    constants::DEFAULT_S3_FORCE_PATH_STYLE
}

#[derive(PartialEq, Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum CredentialConfig {
    Plain { plain: String },
    Path { path: String },
    Env { env: String },
}

impl Into<String> for CredentialConfig {
    fn into(self) -> String {
        match self {
            CredentialConfig::Plain { plain } => plain,
            CredentialConfig::Path { path } => std::fs::read(path.clone())
                .unwrap_or_else(|_| panic!("Unable to read credential file \"{path}\""))
                .try_into()
                .unwrap_or_else(|_| panic!("Unable to read content of credential file \"{path}\"")),
            CredentialConfig::Env { env } => std::env::var(env.clone()).unwrap_or_else(|_| {
                panic!("Missing or unprocessable environment variable \"{env}\"")
            }),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BucketConfig {
    pub endpoint_url: String,
    pub bucket_name: String,
    pub access_key: CredentialConfig,
    pub secret_key: CredentialConfig,
    #[serde(default = "default_region")]
    pub region: String,
    #[serde(default = "default_force_path_style")]
    pub force_path_style: bool,
    pub presign_expiry_secs: Option<u64>,
    #[serde(default)]
    pub proxy: bool,
}

fn default_listen() -> String {
    format!(
        "{}:{}",
        constants::DEFAULT_BIND_ADDR,
        constants::DEFAULT_BIND_PORT
    )
}

fn default_presign_expiry_secs() -> u64 {
    constants::DEFAULT_PRESIGN_EXPIRY
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_listen")]
    pub listen: String,
    #[serde(default = "default_presign_expiry_secs")]
    pub presign_expiry_secs: u64,
    pub buckets: HashMap<String, BucketConfig>,
}

impl AppConfig {
    pub fn load() -> eyre::Result<Self> {
        let path = std::env::var(constants::MEDIA_SERVER_CONFIG_PATH)
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(constants::DEFAULT_CONFIG_PATH));

        let contents = std::fs::read_to_string(&path)?;

        let config: AppConfig = serde_yaml::from_str(&contents)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests;

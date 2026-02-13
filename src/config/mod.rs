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

#[derive(Debug, Deserialize)]
pub struct BucketConfig {
    pub endpoint_url: String,
    pub bucket_name: String,
    pub access_key: String,
    pub secret_key: String,
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
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = std::env::var(constants::CELIA_CONFIG_PATH)
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(constants::DEFAULT_CONFIG_PATH));

        let contents = std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read config file {}: {e}", path.display()))?;

        let config: AppConfig = serde_yaml::from_str(&contents)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests;

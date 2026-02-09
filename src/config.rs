use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

pub const CONFIG_FILE: &str = ".config/vault-conductor/config.yaml";

#[derive(Debug, Deserialize)]
pub struct Config {
    pub bws_access_token: String,
    pub bw_secret_ids: Vec<String>,
    #[serde(default)]
    pub bw_server_endpoint: Option<String>,
}

impl Config {
    pub fn load(config_file: &Option<String>) -> Result<Self> {
        let config_path = match config_file {
            Some(file) => PathBuf::from(file),
            None => Self::get_config_path()?,
        };

        // Try to load from config file first
        if config_path.exists() {
            let config_content = std::fs::read_to_string(&config_path).with_context(|| {
                format!("Failed to read config file: {}", config_path.display())
            })?;

            let mut config: Config = serde_yaml::from_str(&config_content)
                .context("Failed to parse config file as YAML")?;

            // Environment variable overrides config file for server endpoint
            if let Ok(endpoint) = std::env::var("BW_SERVER_ENDPOINT") {
                config.bw_server_endpoint = Some(endpoint);
            }

            Ok(config)
        } else {
            // Fallback to environment variables
            let bws_access_token = std::env::var("BWS_ACCESS_TOKEN").with_context(|| {
                format!(
                    "Config file not found at {} and BWS_ACCESS_TOKEN environment variable is not set",
                    config_path.display()
                )
            })?;

            let bw_secret_ids_string = std::env::var("BW_SECRET_IDS").with_context(|| {
                format!(
                    "Config file not found at {} and BW_SECRET_IDS environment variable is not set",
                    config_path.display()
                )
            })?;

            let bw_secret_ids: Vec<String> = bw_secret_ids_string
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();

            // Optional server endpoint from environment
            let bw_server_endpoint = std::env::var("BW_SERVER_ENDPOINT").ok();

            Ok(Config {
                bws_access_token,
                bw_secret_ids,
                bw_server_endpoint,
            })
        }
    }

    fn get_config_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir().context("Unable to determine home directory")?;
        Ok(home_dir.join(CONFIG_FILE))
    }

    pub fn get_api_url(&self) -> String {
        self.bw_server_endpoint
            .as_ref()
            .map(|host| {
                if host == "bitwarden.com" || host == "bitwarden.eu" {
                    // Cloud instance - use subdomain pattern
                    format!("https://api.{}", host)
                } else {
                    // Self-hosted - use path pattern
                    format!("https://{}/api", host)
                }
            })
            .unwrap_or_else(|| "https://api.bitwarden.com".to_string())
    }

    pub fn get_identity_url(&self) -> String {
        self.bw_server_endpoint
            .as_ref()
            .map(|host| {
                if host == "bitwarden.com" || host == "bitwarden.eu" {
                    // Cloud instance - use subdomain pattern
                    format!("https://identity.{}", host)
                } else {
                    // Self-hosted - use path pattern
                    format!("https://{}/identity", host)
                }
            })
            .unwrap_or_else(|| "https://identity.bitwarden.com".to_string())
    }
}

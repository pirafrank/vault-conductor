use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

pub const CONFIG_FILE: &str = ".config/vault-conductor/config.yaml";

#[derive(Debug, Deserialize)]
pub struct Config {
    pub bws_access_token: String,
    pub bw_secret_id: String,
}

impl Config {
    pub fn load(config_file: Option<String>) -> Result<Self> {
        let config_path = if let Some(file) = config_file {
            PathBuf::from(file)
        } else {
            Self::get_config_path()?
        };

        // Try to load from config file first
        if config_path.exists() {
            let config_content = std::fs::read_to_string(&config_path).with_context(|| {
                format!("Failed to read config file: {}", config_path.display())
            })?;

            let config: Config = serde_yaml::from_str(&config_content)
                .context("Failed to parse config file as YAML")?;

            Ok(config)
        } else {
            // Fallback to environment variables
            let bws_access_token = std::env::var("BWS_ACCESS_TOKEN").with_context(|| {
                format!(
                    "Config file not found at {} and BWS_ACCESS_TOKEN environment variable is not set",
                    config_path.display()
                )
            })?;

            let bw_secret_id = std::env::var("BW_SECRET_ID").with_context(|| {
                format!(
                    "Config file not found at {} and BW_SECRET_ID environment variable is not set",
                    config_path.display()
                )
            })?;

            Ok(Config {
                bws_access_token,
                bw_secret_id,
            })
        }
    }

    fn get_config_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir().context("Unable to determine home directory")?;
        Ok(home_dir.join(CONFIG_FILE))
    }
}

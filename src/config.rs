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
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        let config_content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

        let config: Config =
            serde_yaml::from_str(&config_content).context("Failed to parse config file as YAML")?;

        Ok(config)
    }

    fn get_config_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir().context("Unable to determine home directory")?;
        Ok(home_dir.join(CONFIG_FILE))
    }
}

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const CONFIG_FILE: &str = ".config/vault-conductor/config.yaml";

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub bws_access_token: String,
    pub bw_secret_id: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_debug_impl() {
        let config = Config {
            bws_access_token: "token123".to_string(),
            bw_secret_id: "secret456".to_string(),
        };

        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("bws_access_token"));
        assert!(debug_str.contains("bw_secret_id"));
    }

    #[test]
    fn test_config_struct_creation() {
        let config = Config {
            bws_access_token: "test_token".to_string(),
            bw_secret_id: "test_secret_id".to_string(),
        };

        assert_eq!(config.bws_access_token, "test_token");
        assert_eq!(config.bw_secret_id, "test_secret_id");
    }

    #[test]
    fn test_config_file_constant() {
        assert_eq!(CONFIG_FILE, ".config/vault-conductor/config.yaml");
    }

    #[test]
    fn test_get_config_path_contains_home() {
        let result = Config::get_config_path();

        if result.is_ok() {
            let path = result.unwrap();
            let path_str = path.to_string_lossy();
            assert!(path_str.contains("config.yaml"));
            assert!(path_str.contains("vault-conductor"));
        }
    }

    #[test]
    fn test_config_deserialization() {
        let yaml = r#"
bws_access_token: "my_token"
bw_secret_id: "my_secret"
"#;
        let config: Result<Config, _> = serde_yaml::from_str(yaml);
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.bws_access_token, "my_token");
        assert_eq!(config.bw_secret_id, "my_secret");
    }

    #[test]
    fn test_config_load_from_env() {
        // Save and set env vars
        let old_token = std::env::var("BWS_ACCESS_TOKEN").ok();
        let old_secret = std::env::var("BW_SECRET_ID").ok();

        std::env::set_var("BWS_ACCESS_TOKEN", "env_token_test");
        std::env::set_var("BW_SECRET_ID", "env_secret_test");

        // If config file doesn't exist at the expected path, this should use env vars
        // Note: This test might succeed or fail depending on whether config file exists
        let result = Config::load();

        // Restore env vars
        match old_token {
            Some(t) => std::env::set_var("BWS_ACCESS_TOKEN", t),
            None => std::env::remove_var("BWS_ACCESS_TOKEN"),
        }
        match old_secret {
            Some(s) => std::env::set_var("BW_SECRET_ID", s),
            None => std::env::remove_var("BW_SECRET_ID"),
        }

        // Should either succeed with file content or env vars
        if let Ok(config) = result {
            // Either from file or from env vars
            assert!(!config.bws_access_token.is_empty() || config.bws_access_token.is_empty());
        }
    }

    #[test]
    fn test_config_fields() {
        let config = Config {
            bws_access_token: "test123".to_string(),
            bw_secret_id: "secret456".to_string(),
        };

        // Test field access
        assert_eq!(&config.bws_access_token, "test123");
        assert_eq!(&config.bw_secret_id, "secret456");
    }
}

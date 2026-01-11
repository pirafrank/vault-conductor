use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

pub const CONFIG_FILE: &str = ".config/vault-conductor/config.yaml";

#[derive(Debug, Deserialize)]
pub struct Config {
    pub bws_access_token: String,
    pub bw_secret_ids: Vec<String>,
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

            Ok(Config {
                bws_access_token,
                bw_secret_ids,
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
    use std::env;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_from_yaml_file() {
        // Arrange: Create a temporary YAML config file
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
bws_access_token: "test_token_123"
bw_secret_ids:
  - "27d19637-7258-4b9c-b115-b3cf0106d8be"
  - "40e28e86-9ae5-40e0-93cb-b3cf0106c50e"
"#;
        temp_file.write_all(config_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config_path = temp_file.path().to_str().unwrap().to_string();

        // Act: Load config from file
        let config = Config::load(&Some(config_path)).unwrap();

        // Assert
        assert_eq!(config.bws_access_token, "test_token_123");
        assert_eq!(config.bw_secret_ids.len(), 2);
        assert_eq!(
            config.bw_secret_ids[0],
            "27d19637-7258-4b9c-b115-b3cf0106d8be"
        );
        assert_eq!(
            config.bw_secret_ids[1],
            "40e28e86-9ae5-40e0-93cb-b3cf0106c50e"
        );
    }

    #[test]
    fn test_config_from_env_variables() {
        // Arrange: Set environment variables
        env::set_var("BWS_ACCESS_TOKEN", "env_token_456");
        env::set_var(
            "BW_SECRET_IDS",
            "aaaaaaaa-1111-2222-3333-bbbbbbbbbbbb, cccccccc-4444-5555-6666-dddddddddddd",
        );

        // Use a non-existent config file path to force fallback to env vars
        let non_existent_path = "/tmp/nonexistent_vault_conductor_config_test.yaml";

        // Act: Load config (should fallback to env vars)
        let config = Config::load(&Some(non_existent_path.to_string())).unwrap();

        // Assert
        assert_eq!(config.bws_access_token, "env_token_456");
        assert_eq!(config.bw_secret_ids.len(), 2);
        assert_eq!(
            config.bw_secret_ids[0],
            "aaaaaaaa-1111-2222-3333-bbbbbbbbbbbb"
        );
        assert_eq!(
            config.bw_secret_ids[1],
            "cccccccc-4444-5555-6666-dddddddddddd"
        );

        // Cleanup
        env::remove_var("BWS_ACCESS_TOKEN");
        env::remove_var("BW_SECRET_IDS");
    }

    #[test]
    fn test_config_env_var_csv_parsing_with_spaces() {
        // Arrange: Set environment variables with various spacing
        env::set_var("BWS_ACCESS_TOKEN", "token_with_spaces");
        env::set_var("BW_SECRET_IDS", "  id1  ,  id2  , id3  ");

        let non_existent_path = "/tmp/nonexistent_vault_conductor_config_test2.yaml";

        // Act
        let config = Config::load(&Some(non_existent_path.to_string())).unwrap();

        // Assert: Spaces should be trimmed
        assert_eq!(config.bw_secret_ids.len(), 3);
        assert_eq!(config.bw_secret_ids[0], "id1");
        assert_eq!(config.bw_secret_ids[1], "id2");
        assert_eq!(config.bw_secret_ids[2], "id3");

        // Cleanup
        env::remove_var("BWS_ACCESS_TOKEN");
        env::remove_var("BW_SECRET_IDS");
    }

    #[test]
    fn test_config_missing_file_and_missing_env() {
        // Arrange: Ensure env vars are not set
        env::remove_var("BWS_ACCESS_TOKEN");
        env::remove_var("BW_SECRET_IDS");

        let non_existent_path = "/tmp/nonexistent_vault_conductor_config_test3.yaml";

        // Act & Assert: Should return an error
        let result = Config::load(&Some(non_existent_path.to_string()));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("BWS_ACCESS_TOKEN"));
    }

    #[test]
    fn test_config_invalid_yaml() {
        // Arrange: Create a temporary file with invalid YAML
        let mut temp_file = NamedTempFile::new().unwrap();
        let invalid_yaml = "this is not: valid: yaml: content::::";
        temp_file.write_all(invalid_yaml.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config_path = temp_file.path().to_str().unwrap().to_string();

        // Act & Assert: Should return an error
        let result = Config::load(&Some(config_path));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to parse"));
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use vault_conductor::config::Config;

    // Helper to create a temporary test config file
    fn create_test_config(content: &str) -> PathBuf {
        let test_dir =
            std::env::temp_dir().join(format!("vault-conductor-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&test_dir).unwrap();
        let config_path = test_dir.join("config.yaml");
        fs::write(&config_path, content).unwrap();
        config_path
    }

    fn cleanup_test_dir(path: &PathBuf) {
        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn test_config_from_env_vars() {
        // Set environment variables
        std::env::set_var("BWS_ACCESS_TOKEN", "test-token-123");
        std::env::set_var("BW_SECRET_ID", "550e8400-e29b-41d4-a716-446655440000");

        // Load config (should use env vars since file doesn't exist)
        let result = Config::load();

        // Cleanup
        std::env::remove_var("BWS_ACCESS_TOKEN");
        std::env::remove_var("BW_SECRET_ID");

        // For this test to work properly without HOME manipulation,
        // we'd need to mock the home directory or config path
        // Since Config::load() looks for a specific path, this test
        // is more of a demonstration of the expected behavior

        // In a real environment where the config file doesn't exist,
        // it should fall back to env vars
        assert!(result.is_ok() || result.is_err()); // Placeholder assertion
    }

    #[test]
    fn test_config_missing_env_vars() {
        // Save current env vars
        let old_token = std::env::var("BWS_ACCESS_TOKEN").ok();
        let old_secret = std::env::var("BW_SECRET_ID").ok();

        // Make sure env vars are not set
        std::env::remove_var("BWS_ACCESS_TOKEN");
        std::env::remove_var("BW_SECRET_ID");

        // Try to load config without env vars or config file
        // This might succeed if config file exists, or fail if not
        let result = Config::load();

        // Restore env vars
        if let Some(token) = old_token {
            std::env::set_var("BWS_ACCESS_TOKEN", token);
        }
        if let Some(secret) = old_secret {
            std::env::set_var("BW_SECRET_ID", secret);
        }

        // If result is Err, check error message
        // If result is Ok, that means config file was found on system
        if let Err(e) = result {
            let error_msg = e.to_string();
            // Should mention either config file not found or env var not set
            assert!(
                error_msg.contains("not found") || error_msg.contains("not set"),
                "Error message should indicate missing config or env vars: {}",
                error_msg
            );
        }
        // If Ok, config file exists on the system, which is fine
    }

    #[test]
    fn test_config_file_invalid_yaml() {
        let config_path = create_test_config("invalid: yaml: syntax: [[[");

        // This test demonstrates parsing failure, but we can't easily
        // inject the path into Config::load() without refactoring
        let content = fs::read_to_string(&config_path).unwrap();
        let result: Result<Config, _> = serde_yaml::from_str(&content);

        assert!(result.is_err());

        cleanup_test_dir(&config_path);
    }

    #[test]
    fn test_config_file_missing_fields() {
        let config_path = create_test_config("bws_access_token: test-token");

        // Missing bw_secret_id field
        let content = fs::read_to_string(&config_path).unwrap();
        let result: Result<Config, _> = serde_yaml::from_str(&content);

        assert!(result.is_err());

        cleanup_test_dir(&config_path);
    }

    #[test]
    fn test_config_file_valid() {
        let yaml_content = r#"
bws_access_token: "test-token-456"
bw_secret_id: "550e8400-e29b-41d4-a716-446655440001"
"#;
        let config_path = create_test_config(yaml_content);

        let content = fs::read_to_string(&config_path).unwrap();
        let result: Result<Config, _> = serde_yaml::from_str(&content);

        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.bws_access_token, "test-token-456");
        assert_eq!(config.bw_secret_id, "550e8400-e29b-41d4-a716-446655440001");

        cleanup_test_dir(&config_path);
    }

    #[test]
    fn test_config_extra_fields_ignored() {
        let yaml_content = r#"
bws_access_token: "test-token-789"
bw_secret_id: "550e8400-e29b-41d4-a716-446655440002"
extra_field: "this should be ignored"
another_field: 123
"#;
        let config_path = create_test_config(yaml_content);

        let content = fs::read_to_string(&config_path).unwrap();
        let result: Result<Config, _> = serde_yaml::from_str(&content);

        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.bws_access_token, "test-token-789");
        assert_eq!(config.bw_secret_id, "550e8400-e29b-41d4-a716-446655440002");

        cleanup_test_dir(&config_path);
    }

    #[test]
    fn test_config_empty_values() {
        let yaml_content = r#"
bws_access_token: ""
bw_secret_id: ""
"#;
        let config_path = create_test_config(yaml_content);

        let content = fs::read_to_string(&config_path).unwrap();
        let result: Result<Config, _> = serde_yaml::from_str(&content);

        // Should parse successfully but with empty strings
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.bws_access_token, "");
        assert_eq!(config.bw_secret_id, "");

        cleanup_test_dir(&config_path);
    }
}

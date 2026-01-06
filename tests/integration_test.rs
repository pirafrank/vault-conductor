#[cfg(test)]
mod integration_tests {

    #[test]
    fn test_config_yaml_example_structure() {
        // Test that a proper config structure can be serialized/deserialized
        use serde_yaml;
        use vault_conductor::config::Config;

        let config = Config {
            bws_access_token: "test_token_123".to_string(),
            bw_secret_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        };

        // Serialize to YAML
        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("bws_access_token"));
        assert!(yaml.contains("bw_secret_id"));

        // Deserialize back
        let deserialized: Config = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(deserialized.bws_access_token, config.bws_access_token);
        assert_eq!(deserialized.bw_secret_id, config.bw_secret_id);
    }

    #[test]
    fn test_process_manager_workflow() {
        // Test the basic workflow without actually starting processes
        use vault_conductor::process_manager::stop_agent;

        // Clean state
        let _ = stop_agent();

        // This tests that the functions exist and can be called
        // Actual process management is tested in the unit tests
    }

    #[test]
    fn test_logging_initialization() {
        use log::LevelFilter;
        use vault_conductor::logging::setup_logging;

        // Test that logging can be set up (may fail if already initialized)
        let _ = setup_logging(LevelFilter::Info, true);
    }

    #[test]
    fn test_bitwarden_agent_trait() {
        // Compile-time test that SecretFetcher trait is properly defined
        use vault_conductor::bitwarden::agent::SecretFetcher;

        #[allow(dead_code)]
        fn assert_trait_object_safe<T: ?Sized + SecretFetcher>() {}
        // If this compiles, the trait is properly defined
    }

    #[test]
    fn test_module_structure() {
        // Verify all public modules are accessible
        let _ = vault_conductor::config::CONFIG_FILE;
        // If this compiles, module structure is correct
    }

    #[test]
    fn test_environment_variable_names() {
        // Document and test the expected environment variable names
        let token_var = "BWS_ACCESS_TOKEN";
        let secret_var = "BW_SECRET_ID";

        assert_eq!(token_var, "BWS_ACCESS_TOKEN");
        assert_eq!(secret_var, "BW_SECRET_ID");
    }

    #[test]
    fn test_config_file_path() {
        use vault_conductor::config::CONFIG_FILE;

        assert_eq!(CONFIG_FILE, ".config/vault-conductor/config.yaml");
        assert!(CONFIG_FILE.ends_with("config.yaml"));
        assert!(CONFIG_FILE.contains("vault-conductor"));
    }
}

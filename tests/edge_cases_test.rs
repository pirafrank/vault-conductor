#[cfg(test)]
mod edge_cases {
    use std::env;
    use vault_conductor::config::Config;

    #[test]
    fn test_config_with_special_characters() {
        let yaml = r#"
bws_access_token: "token!@#$%^&*()_+-=[]{}|;':,.<>?"
bw_secret_id: "550e8400-e29b-41d4-a716-446655440000"
"#;
        let config: Result<Config, _> = serde_yaml::from_str(yaml);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert!(config.bws_access_token.contains("!@#$"));
    }

    #[test]
    fn test_config_with_very_long_values() {
        let long_token = "a".repeat(1000);
        let yaml = format!(
            r#"
bws_access_token: "{}"
bw_secret_id: "550e8400-e29b-41d4-a716-446655440000"
"#,
            long_token
        );
        let config: Result<Config, _> = serde_yaml::from_str(&yaml);
        assert!(config.is_ok());
        assert_eq!(config.unwrap().bws_access_token.len(), 1000);
    }

    #[test]
    fn test_config_with_unicode() {
        let yaml = r#"
bws_access_token: "token_测试_тест_🔑"
bw_secret_id: "550e8400-e29b-41d4-a716-446655440000"
"#;
        let config: Result<Config, _> = serde_yaml::from_str(yaml);
        assert!(config.is_ok());
        assert!(config.unwrap().bws_access_token.contains("🔑"));
    }

    #[test]
    fn test_config_whitespace_handling() {
        let yaml = r#"
bws_access_token:    "  token_with_spaces  "
bw_secret_id:    "550e8400-e29b-41d4-a716-446655440000"
"#;
        let config: Result<Config, _> = serde_yaml::from_str(yaml);
        assert!(config.is_ok());
    }

    #[test]
    fn test_config_null_values() {
        // YAML null handling
        let yaml = r#"
bws_access_token: null
bw_secret_id: "550e8400-e29b-41d4-a716-446655440000"
"#;
        let config: Result<Config, _> = serde_yaml::from_str(yaml);
        // Depending on serde settings, this might succeed with null or fail
        // We just check it doesn't panic
        let _ = config;
    }

    #[test]
    fn test_process_manager_edge_cases() {
        use std::sync::Mutex;
        use vault_conductor::process_manager::write_pid;

        static LOCK: Mutex<()> = Mutex::new(());
        let _lock = LOCK.lock().unwrap();

        // Test with extreme PID values
        let _ = write_pid(1); // Minimum valid PID
        let _ = write_pid(99999); // Large PID
    }

    #[test]
    fn test_logging_with_all_levels() {
        use log::LevelFilter;
        use vault_conductor::logging::setup_logging;

        // Test all log levels exist and can be used
        let levels = vec![
            LevelFilter::Off,
            LevelFilter::Error,
            LevelFilter::Warn,
            LevelFilter::Info,
            LevelFilter::Debug,
            LevelFilter::Trace,
        ];

        for level in levels {
            let _ = setup_logging(level, true);
        }
    }

    #[test]
    fn test_uuid_parsing() {
        use uuid::Uuid;

        // Valid UUID
        let valid = "550e8400-e29b-41d4-a716-446655440000";
        assert!(Uuid::parse_str(valid).is_ok());

        // Invalid UUID
        let invalid = "not-a-uuid";
        assert!(Uuid::parse_str(invalid).is_err());
    }

    #[test]
    fn test_concurrent_config_access() {
        use std::thread;

        let handles: Vec<_> = (0..10)
            .map(|_| {
                thread::spawn(|| {
                    let yaml = r#"
bws_access_token: "concurrent_test"
bw_secret_id: "550e8400-e29b-41d4-a716-446655440000"
"#;
                    let _config: Result<Config, _> = serde_yaml::from_str(yaml);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_env_var_priority() {
        // Test environment variable behavior
        env::set_var("TEST_VAR_123", "value123");
        assert_eq!(env::var("TEST_VAR_123").unwrap(), "value123");
        env::remove_var("TEST_VAR_123");
        assert!(env::var("TEST_VAR_123").is_err());
    }
}

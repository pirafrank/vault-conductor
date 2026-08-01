#[cfg(all(test, unix))]
mod tests {
    use crate::config::tests::common::{create_config, test_path};
    use crate::config::Config;

    use std::fs;
    use std::sync::{Mutex, OnceLock};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    #[test]
    fn custom_config_path_is_validated() {
        let path = test_path("custom");
        create_config(&path, 0o644);

        assert!(Config::load(&Some(path.to_string_lossy().into_owned())).is_ok());

        fs::remove_file(path).expect("failed to remove test config");
    }

    #[test]
    fn missing_config_uses_environment_variables() {
        let _lock = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let path = test_path("environment");
        std::env::set_var("BWS_ACCESS_TOKEN", "environment-token");
        std::env::set_var("BW_SECRET_IDS", "environment-secret-id");

        let config = Config::load(&Some(path.to_string_lossy().into_owned()))
            .expect("environment-only configuration should load");

        assert_eq!(config.bws_access_token, "environment-token");
        assert_eq!(config.bw_secret_ids, vec!["environment-secret-id"]);
        std::env::remove_var("BWS_ACCESS_TOKEN");
        std::env::remove_var("BW_SECRET_IDS");
    }
}

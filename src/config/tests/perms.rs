#[cfg(all(test, unix))]
mod tests {
    use crate::config::tests::common::{create_config, test_path};
    use crate::config::{inspect_config_permissions, Config};

    use std::fs;

    #[test]
    fn exact_0600_permissions_are_accepted() {
        let path = test_path("0600");
        create_config(&path, 0o600);

        assert_eq!(inspect_config_permissions(&path).unwrap(), None);
        assert!(Config::load(&Some(path.to_string_lossy().into_owned())).is_ok());

        fs::remove_file(path).expect("failed to remove test config");
    }

    #[test]
    fn non_0600_permissions_are_warned_about_and_accepted() {
        for mode in [0o644, 0o660, 0o601] {
            let path = test_path(&format!("{mode:o}"));
            create_config(&path, mode);

            assert_eq!(inspect_config_permissions(&path).unwrap(), Some(mode));
            assert!(Config::load(&Some(path.to_string_lossy().into_owned())).is_ok());

            fs::remove_file(path).expect("failed to remove test config");
        }
    }

    #[test]
    fn permission_inspection_failures_return_an_error() {
        let path = test_path("missing");

        let error = inspect_config_permissions(&path).expect_err("missing metadata should fail");

        assert!(error.to_string().contains("Failed to inspect permissions"));
    }
}

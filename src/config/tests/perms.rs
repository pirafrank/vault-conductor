#[cfg(all(test, unix))]
mod tests {
    use crate::config::tests::common::{create_config, test_path};
    use crate::config::{inspect_config_permissions, Config};

    use std::fs;

    #[test]
    fn owner_only_permissions_are_accepted() {
        for mode in [0o400, 0o600] {
            let path = test_path(&format!("{mode:o}"));
            create_config(&path, mode);

            assert_eq!(inspect_config_permissions(&path).unwrap(), None);
            assert!(Config::load(&Some(path.to_string_lossy().into_owned())).is_ok());

            fs::remove_file(path).expect("failed to remove test config");
        }
    }

    #[test]
    fn permissions_wider_than_owner_only_are_rejected() {
        for mode in [0o700, 0o640, 0o660, 0o601, 0o604, 0o2600, 0o4600] {
            let path = test_path(&format!("{mode:o}"));
            create_config(&path, mode);

            let error = inspect_config_permissions(&path)
                .expect_err("config permissions wider than owner-only should fail");
            assert!(error.to_string().contains(&format!("{mode:04o}")));
            assert!(Config::load(&Some(path.to_string_lossy().into_owned())).is_err());

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

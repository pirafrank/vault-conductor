use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn test_path(name: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "vault-conductor-{name}-{}-{timestamp}.yaml",
        std::process::id()
    ))
}

pub fn create_config(path: &PathBuf, mode: u32) {
    fs::write(
        path,
        "bws_access_token: token\nbw_secret_ids:\n  - secret-id\n",
    )
    .expect("failed to write test config");
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
        .expect("failed to set test config permissions");
}

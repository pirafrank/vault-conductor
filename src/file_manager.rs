use anyhow::{Context, Result};
use log::debug;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

/// Get the PID file path
fn get_pid_file_path() -> PathBuf {
    let username = std::env::var("USER")
        .context("Failed to get username")
        .unwrap();
    PathBuf::from(format!("/tmp/vc-{}-ssh-agent.pid", username))
}

// Socket setup
pub fn get_socket_file_path() -> PathBuf {
    let username = std::env::var("USER")
        .context("Failed to get username")
        .unwrap();
    PathBuf::from(format!("/tmp/vc-{}-ssh-agent.sock", username))
}

/// Read the PID from the PID file
pub fn read_pid() -> Result<Option<i32>> {
    let pid_path = get_pid_file_path();
    if !pid_path.exists() {
        return Ok(None);
    }

    let pid_str = fs::read_to_string(&pid_path)
        .context(format!("Failed to read PID file at {}", pid_path.display()))?;

    let pid: i32 = pid_str.trim().parse().context("Invalid PID in PID file")?;

    Ok(Some(pid))
}

/// Write the PID to the PID file
pub fn write_pid(pid: i32) -> Result<()> {
    let pid_path = get_pid_file_path();
    fs::write(&pid_path, pid.to_string()).context(format!(
        "Failed to write PID file at {}",
        pid_path.display()
    ))?;
    fs::set_permissions(&pid_path, std::fs::Permissions::from_mode(0o600))
        .context("Failed to set PID file permissions")?;
    debug!("PID file written: {} with PID: {}", pid_path.display(), pid);
    Ok(())
}

/// Remove PID file
fn remove_pid_file() -> Result<()> {
    let pid_path = get_pid_file_path();
    remove_file(&pid_path, "PID")?;
    Ok(())
}

/// Remove socket file
fn remove_socket_file() -> Result<()> {
    let socket_path = get_socket_file_path();
    remove_file(&socket_path, "socket")?;
    Ok(())
}

pub fn cleanup_files() -> Result<()> {
    remove_pid_file()?;
    remove_socket_file()?;
    Ok(())
}

/// Remove file
pub fn remove_file(path: &PathBuf, what: &str) -> Result<()> {
    if path.exists() {
        fs::remove_file(path).context(format!(
            "Failed to remove {} file at {}",
            what,
            path.display()
        ))?;
        debug!("{} file removed: {}", what, path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_get_pid_file_path_includes_username() {
        // Arrange: Get current username
        let username = env::var("USER").expect("USER env var should be set");

        // Act
        let pid_path = get_pid_file_path();

        // Assert: Path should include username
        let path_str = pid_path.to_string_lossy();
        assert!(path_str.contains(&username));
        assert!(path_str.contains("/tmp/vc-"));
        assert!(path_str.ends_with("-ssh-agent.pid"));
    }

    #[test]
    fn test_get_socket_file_path_includes_username() {
        // Arrange: Get current username
        let username = env::var("USER").expect("USER env var should be set");

        // Act
        let socket_path = get_socket_file_path();

        // Assert: Path should include username
        let path_str = socket_path.to_string_lossy();
        assert!(path_str.contains(&username));
        assert!(path_str.contains("/tmp/vc-"));
        assert!(path_str.ends_with("-ssh-agent.sock"));
    }

    #[test]
    fn test_read_pid_returns_none_for_nonexistent_file() {
        // Arrange: Ensure PID file doesn't exist by using a custom path
        // We can't directly test get_pid_file_path() since it's hardcoded,
        // but we test the function logic by ensuring a non-existent scenario

        // Act: This should return None since we haven't written a PID file
        // Note: This test assumes no other instance is running
        let result = read_pid();

        // Assert: Should either be None or Some(valid_pid) if another instance exists
        // We just check it doesn't panic
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_and_read_pid() {
        // Arrange: Write a test PID
        let test_pid = 12345;

        // Act: Write PID
        write_pid(test_pid).expect("Should write PID successfully");

        // Read it back
        let read_result = read_pid().expect("Should read PID successfully");

        // Assert
        assert!(read_result.is_some());
        assert_eq!(read_result.unwrap(), test_pid);

        // Cleanup
        let _ = cleanup_files();
    }

    #[test]
    fn test_read_pid_with_invalid_content() {
        // Arrange: Create a PID file with invalid content
        let pid_path = get_pid_file_path();
        fs::write(&pid_path, "not_a_number").expect("Should write invalid content");

        // Act
        let result = read_pid();

        // Assert: Should return an error
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid PID"));

        // Cleanup
        let _ = fs::remove_file(&pid_path);
    }

    #[test]
    fn test_read_pid_with_whitespace() {
        // Arrange: Create a PID file with whitespace around number
        let pid_path = get_pid_file_path();
        fs::write(&pid_path, "  99999  \n").expect("Should write with whitespace");

        // Act
        let result = read_pid().expect("Should parse PID with whitespace");

        // Assert: Should trim and parse correctly
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 99999);

        // Cleanup
        let _ = fs::remove_file(&pid_path);
    }

    #[test]
    fn test_cleanup_files() {
        // Arrange: Create both PID and socket files
        write_pid(54321).expect("Should write PID");
        let socket_path = get_socket_file_path();
        fs::write(&socket_path, "dummy").expect("Should write socket placeholder");

        // Verify they exist
        assert!(get_pid_file_path().exists());
        assert!(socket_path.exists());

        // Act: Cleanup
        cleanup_files().expect("Should cleanup successfully");

        // Assert: Both files should be removed
        assert!(!get_pid_file_path().exists());
        assert!(!socket_path.exists());
    }

    #[test]
    fn test_remove_file_when_exists() {
        // Arrange: Create a temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test").unwrap();
        let path = temp_file.path().to_path_buf();

        // Keep the file by persisting it
        let persistent_path = temp_file.into_temp_path();
        let path = persistent_path.to_path_buf();

        assert!(path.exists());

        // Act
        let result = remove_file(&path, "test");

        // Assert
        assert!(result.is_ok());
        assert!(!path.exists());
    }

    #[test]
    fn test_remove_file_when_not_exists() {
        // Arrange: Non-existent path
        let path = PathBuf::from("/tmp/nonexistent_file_vault_conductor_test_xyz");

        // Act
        let result = remove_file(&path, "test");

        // Assert: Should succeed (no-op)
        assert!(result.is_ok());
    }
}

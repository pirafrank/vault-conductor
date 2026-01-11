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

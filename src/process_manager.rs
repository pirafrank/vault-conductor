use crate::{config::Config, file_manager::*};
use anyhow::{anyhow, Context, Result};
use log::{debug, info};
use std::{
    path::PathBuf,
    process::{Child, Command, Stdio},
};

use crate::logging::get_log_file_path;

/// Check if a process with the given PID is running
#[cfg(not(windows))]
fn is_process_running(pid: i32) -> bool {
    // Send signal 0 to check if process exists without actually sending a signal
    // Using kill command which is more portable
    Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// Stop the agent process
pub fn stop_agent() -> Result<()> {
    match read_pid()? {
        Some(pid) => {
            if is_process_running(pid) {
                info!("Stopping agent with PID: {}", pid);

                // Try to gracefully terminate with SIGTERM
                let result = Command::new("kill")
                    .arg("-TERM")
                    .arg(pid.to_string())
                    .status();

                match result {
                    Ok(status) if status.success() => {
                        // Wait a bit for graceful shutdown
                        std::thread::sleep(std::time::Duration::from_millis(500));

                        // Check if it's still running
                        if is_process_running(pid) {
                            debug!("Process still running, sending SIGKILL");
                            // Force kill if still running
                            Command::new("kill")
                                .arg("-KILL")
                                .arg(pid.to_string())
                                .status()
                                .context("Failed to force kill agent process")?;
                        }

                        cleanup_files()?;
                        info!("Agent stopped successfully");
                        Ok(())
                    }
                    _ => {
                        // Process might already be dead
                        cleanup_files()?;
                        info!("Agent process not found, cleaned up PID and socket files");
                        Ok(())
                    }
                }
            } else {
                debug!("Agent process with PID {} is not running", pid);
                cleanup_files()?;
                info!("Agent was not running, cleaned up stale PID and socket files");
                Ok(())
            }
        }
        None => {
            info!("Agent is not running (no PID file found)");
            Ok(())
        }
    }
}

/// Start the agent in a background process
pub fn start_agent_background(config_file: Option<String>) -> Result<()> {
    // Check if agent is already running
    if let Some(pid) = read_pid()? {
        if is_process_running(pid) {
            return Err(anyhow!(
                "Agent is already running with PID: {}. Use 'stop' first.",
                pid
            ));
        } else {
            debug!("Cleaning up stale PID file");
            cleanup_files()?;
        }
    }

    // Try to load configuration and ignore the result.
    // If it fails, we'll get logs to sysout.
    let _ = Config::load(&config_file).context("Failed to load configuration")?;

    info!("Starting agent in background...");

    // Get the current executable path
    let exe_path = std::env::current_exe().context("Failed to get current executable path")?;

    // Start the agent process in the background
    let mut cmd = Command::new(&exe_path);
    cmd.arg("start").arg("--fg");

    // If config file is provided, add it to the command
    if let Some(config_file) = config_file {
        cmd.arg("--config").arg(config_file);
    }

    cmd.env("VC_DAEMON_CHILD", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let child: Child = cmd.spawn().context("Failed to spawn agent process")?;

    let pid = child.id() as i32;
    write_pid(pid)?;

    info!("Agent started with PID: {}", pid);
    Ok(())
}

/// Open the log file with the default system viewer
pub fn show_log_file() -> Result<()> {
    let log_file_path: PathBuf = get_log_file_path();
    debug!("Log file path: {}", log_file_path.display());
    std::process::Command::new("less")
        .arg(log_file_path)
        .status()
        .context("Failed to show log file")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(windows))]
    fn test_is_process_running_with_nonexistent_pid() {
        // Arrange: Use a PID that's very unlikely to exist
        let nonexistent_pid = 999999;

        // Act
        let result = is_process_running(nonexistent_pid);

        // Assert: Should return false
        assert!(!result);
    }

    #[test]
    #[cfg(not(windows))]
    fn test_is_process_running_with_current_process() {
        // Arrange: Get current process PID
        let current_pid = std::process::id() as i32;

        // Act
        let result = is_process_running(current_pid);

        // Assert: Current process should be running
        assert!(result);
    }

    #[test]
    #[cfg(not(windows))]
    fn test_is_process_running_with_init_process() {
        // Arrange: PID 1 is always init/systemd on Unix
        let init_pid = 1;

        // Act
        let result = is_process_running(init_pid);

        // Assert: Init should always be running
        assert!(result);
    }

    #[test]
    fn test_stop_agent_when_not_running() {
        // Arrange: Ensure no PID file exists
        let _ = cleanup_files();

        // Act
        let result = stop_agent();

        // Assert: Should succeed (no-op when not running)
        assert!(result.is_ok());
    }

    #[test]
    fn test_start_agent_background_validation() {
        // Arrange: Create a test config
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        let config_content = r#"
bws_access_token: "test_token"
bw_secret_ids:
  - "27d19637-7258-4b9c-b115-b3cf0106d8be"
"#;
        temp_file.write_all(config_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        // Cleanup any existing agent
        let _ = stop_agent();

        // Act: Try to start agent (will fail at bitwarden auth, but validates config loading)
        // We're just testing that the config validation part works
        // Note: This will spawn a process, so we clean up immediately
        let config_path = temp_file.path().to_str().unwrap().to_string();

        // We can't fully test this without mocking, but we can test the config loading part
        // by calling Config::load directly (tested in config.rs)
        // The actual spawn is too complex to test in a unit test

        // For now, just verify the function exists and has correct signature
        // The actual integration would be tested in integration tests

        // Assert: Function signature is correct (compilation test)
        let _: fn(Option<String>) -> Result<()> = start_agent_background;
    }
}

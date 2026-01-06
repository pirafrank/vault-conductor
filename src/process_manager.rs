use anyhow::{anyhow, Context, Result};
use log::{debug, info};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[cfg(not(windows))]
const PID_FILE: &str = "/tmp/vc-ssh-agent.pid";

/// Get the PID file path
fn pid_file_path() -> PathBuf {
    PathBuf::from(PID_FILE)
}

/// Read the PID from the PID file
fn read_pid() -> Result<Option<i32>> {
    let pid_path = pid_file_path();
    if !pid_path.exists() {
        return Ok(None);
    }

    let pid_str = fs::read_to_string(&pid_path)
        .context(format!("Failed to read PID file at {}", pid_path.display()))?;

    let pid: i32 = pid_str.trim().parse().context("Invalid PID in PID file")?;

    Ok(Some(pid))
}

#[cfg(not(windows))]
/// Write the PID to the PID file
pub fn write_pid(pid: i32) -> Result<()> {
    let pid_path = pid_file_path();
    fs::write(&pid_path, pid.to_string()).context(format!(
        "Failed to write PID file at {}",
        pid_path.display()
    ))?;
    fs::set_permissions(&pid_path, std::fs::Permissions::from_mode(0o600))
        .context("Failed to set PID file permissions")?;
    debug!("PID file written: {} with PID: {}", pid_path.display(), pid);
    Ok(())
}

/// Remove the PID file
fn remove_pid_file() -> Result<()> {
    let pid_path = pid_file_path();
    if pid_path.exists() {
        fs::remove_file(&pid_path).context(format!(
            "Failed to remove PID file at {}",
            pid_path.display()
        ))?;
        debug!("PID file removed: {}", pid_path.display());
    }
    Ok(())
}

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

                        remove_pid_file()?;
                        info!("Agent stopped successfully");
                        Ok(())
                    }
                    _ => {
                        // Process might already be dead
                        remove_pid_file()?;
                        info!("Agent process not found, cleaned up PID file");
                        Ok(())
                    }
                }
            } else {
                debug!("Agent process with PID {} is not running", pid);
                remove_pid_file()?;
                info!("Agent was not running, cleaned up stale PID file");
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
pub fn start_agent_background() -> Result<()> {
    // Check if agent is already running
    if let Some(pid) = read_pid()? {
        if is_process_running(pid) {
            return Err(anyhow!(
                "Agent is already running with PID: {}. Use 'stop-agent' first or 'restart-agent'.",
                pid
            ));
        } else {
            debug!("Cleaning up stale PID file");
            remove_pid_file()?;
        }
    }

    info!("Starting agent in background...");

    // Get the current executable path
    let exe_path = std::env::current_exe().context("Failed to get current executable path")?;

    // Start the agent process in the background
    let child = Command::new(&exe_path)
        .arg("start-agent")
        .arg("--fg")
        .env("VC_DAEMON_CHILD", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to spawn agent process")?;

    let pid = child.id() as i32;
    write_pid(pid)?;

    info!("Agent started with PID: {}", pid);
    Ok(())
}

/// Restart the agent
pub async fn restart_agent() -> Result<()> {
    info!("Restarting agent...");

    // Stop if running
    if let Some(pid) = read_pid()? {
        if is_process_running(pid) {
            stop_agent()?;
        } else {
            remove_pid_file()?;
        }
    }

    // Start the agent
    start_agent_background()?;

    Ok(())
}

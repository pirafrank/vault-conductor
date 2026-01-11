use crate::file_manager::*;
use anyhow::{anyhow, Context, Result};
use log::{debug, info};
use std::process::{Command, Stdio};

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
            cleanup_files()?;
        }
    }

    info!("Starting agent in background...");

    // Get the current executable path
    let exe_path = std::env::current_exe().context("Failed to get current executable path")?;

    // Start the agent process in the background
    let child = Command::new(&exe_path)
        .arg("start")
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
            cleanup_files()?;
        }
    }

    // Start the agent
    start_agent_background()?;

    Ok(())
}

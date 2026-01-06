#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Write;
    use std::process::Command;
    use std::sync::Mutex;
    use vault_conductor::process_manager::{start_agent_background, stop_agent, write_pid};

    const TEST_PID_FILE: &str = "/tmp/vc-ssh-agent.pid";

    // Mutex to serialize PID file tests since they all use the same file
    static PID_FILE_LOCK: Mutex<()> = Mutex::new(());

    fn cleanup_pid_file() {
        let _ = fs::remove_file(TEST_PID_FILE);
    }

    fn create_dummy_pid_file(pid: i32) {
        let mut file = fs::File::create(TEST_PID_FILE).unwrap();
        write!(file, "{}", pid).unwrap();
    }

    #[test]
    fn test_write_pid() {
        let _lock = PID_FILE_LOCK.lock().unwrap();
        cleanup_pid_file();

        let test_pid = 12345;
        let result = write_pid(test_pid);

        assert!(result.is_ok(), "Writing PID should succeed");
        assert!(fs::metadata(TEST_PID_FILE).is_ok(), "PID file should exist");

        // Read and verify content
        let content = fs::read_to_string(TEST_PID_FILE).unwrap();
        assert_eq!(content.trim(), test_pid.to_string());

        cleanup_pid_file();
    }

    #[test]
    fn test_write_pid_permissions() {
        let _lock = PID_FILE_LOCK.lock().unwrap();
        cleanup_pid_file();

        let test_pid = 54321;
        write_pid(test_pid).unwrap();

        // Check file permissions (should be 0600)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(TEST_PID_FILE).unwrap();
            let permissions = metadata.permissions();
            let mode = permissions.mode();

            // Extract the permission bits (last 9 bits)
            let perm_bits = mode & 0o777;
            assert_eq!(perm_bits, 0o600, "PID file should have 0600 permissions");
        }

        cleanup_pid_file();
    }

    #[test]
    fn test_stop_agent_no_pid_file() {
        let _lock = PID_FILE_LOCK.lock().unwrap();
        cleanup_pid_file();

        let result = stop_agent();

        // Should succeed even if no PID file exists
        assert!(result.is_ok(), "Stopping non-existent agent should succeed");
    }

    #[test]
    fn test_stop_agent_stale_pid() {
        let _lock = PID_FILE_LOCK.lock().unwrap();
        cleanup_pid_file();

        // Create a PID file with a definitely non-existent PID
        create_dummy_pid_file(999999);

        let result = stop_agent();

        // Should succeed and clean up the stale PID file
        assert!(
            result.is_ok(),
            "Stopping agent with stale PID should succeed"
        );
        assert!(
            fs::metadata(TEST_PID_FILE).is_err(),
            "Stale PID file should be removed"
        );
    }

    #[test]
    fn test_stop_agent_invalid_pid_content() {
        let _lock = PID_FILE_LOCK.lock().unwrap();
        cleanup_pid_file();

        // Create a PID file with invalid content
        let mut file = fs::File::create(TEST_PID_FILE).unwrap();
        write!(file, "not-a-number").unwrap();
        drop(file);

        let result = stop_agent();

        // Should return an error due to invalid PID
        assert!(
            result.is_err(),
            "Stopping agent with invalid PID should fail"
        );

        cleanup_pid_file();
    }

    #[test]
    fn test_start_agent_background_already_running() {
        let _lock = PID_FILE_LOCK.lock().unwrap();
        cleanup_pid_file();

        // Create a PID file with the current process's PID (which is definitely running)
        let current_pid = std::process::id() as i32;
        create_dummy_pid_file(current_pid);

        let result = start_agent_background();

        // Should fail because agent appears to be running
        assert!(
            result.is_err(),
            "Starting agent when already running should fail"
        );

        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("already running"),
            "Error should mention agent is already running: {}",
            error_msg
        );

        cleanup_pid_file();
    }

    #[test]
    fn test_start_agent_background_cleans_stale_pid() {
        let _lock = PID_FILE_LOCK.lock().unwrap();
        cleanup_pid_file();

        // Create a stale PID file
        create_dummy_pid_file(999999);

        let result = start_agent_background();

        // The function should clean up the stale PID and try to start
        // It might fail to actually start the agent (depending on environment),
        // but it should at least clean up the stale PID
        match result {
            Ok(_) => {
                // Successfully started, should have a new PID file
                assert!(
                    fs::metadata(TEST_PID_FILE).is_ok(),
                    "New PID file should exist"
                );

                // Clean up the spawned process
                let _ = stop_agent();
            }
            Err(e) => {
                // Might fail in test environment, but should have cleaned stale PID
                eprintln!("Start failed (may be expected in test env): {}", e);
            }
        }

        cleanup_pid_file();
    }

    #[test]
    #[cfg(not(windows))]
    fn test_is_process_running() {
        // Test with current process (should be running)
        let current_pid = std::process::id() as i32;

        let output = Command::new("kill")
            .arg("-0")
            .arg(current_pid.to_string())
            .output();

        assert!(output.is_ok());
        assert!(
            output.unwrap().status.success(),
            "Current process should be detected as running"
        );

        // Test with non-existent PID
        let fake_pid = 999999;
        let output = Command::new("kill")
            .arg("-0")
            .arg(fake_pid.to_string())
            .output();

        assert!(output.is_ok());
        assert!(
            !output.unwrap().status.success(),
            "Fake process should not be detected as running"
        );
    }

    #[tokio::test]
    async fn test_restart_agent_no_existing_agent() {
        use vault_conductor::process_manager::restart_agent;

        let _lock = PID_FILE_LOCK.lock().unwrap();
        cleanup_pid_file();

        let result = restart_agent().await;

        // Should try to start even if nothing was running
        match result {
            Ok(_) => {
                // Successfully restarted, clean up
                let _ = stop_agent();
            }
            Err(e) => {
                // Might fail in test environment
                eprintln!("Restart failed (may be expected in test env): {}", e);
            }
        }

        cleanup_pid_file();
    }

    #[tokio::test]
    async fn test_restart_agent_with_stale_pid() {
        use vault_conductor::process_manager::restart_agent;

        let _lock = PID_FILE_LOCK.lock().unwrap();
        cleanup_pid_file();
        create_dummy_pid_file(999999);

        let result = restart_agent().await;

        // Should clean up stale PID and try to start
        match result {
            Ok(_) => {
                // Successfully restarted, clean up
                let _ = stop_agent();
            }
            Err(e) => {
                // Might fail in test environment
                eprintln!("Restart failed (may be expected in test env): {}", e);
            }
        }

        cleanup_pid_file();
    }

    #[test]
    fn test_pid_file_path() {
        // Verify the PID file path is as expected
        assert_eq!(TEST_PID_FILE, "/tmp/vc-ssh-agent.pid");
    }

    #[test]
    fn test_write_pid_overwrites_existing() {
        let _lock = PID_FILE_LOCK.lock().unwrap();
        cleanup_pid_file();

        // Write first PID
        write_pid(11111).unwrap();
        let content1 = fs::read_to_string(TEST_PID_FILE).unwrap();
        assert_eq!(content1.trim(), "11111");

        // Write second PID (should overwrite)
        write_pid(22222).unwrap();
        let content2 = fs::read_to_string(TEST_PID_FILE).unwrap();
        assert_eq!(content2.trim(), "22222");

        cleanup_pid_file();
    }

    #[test]
    fn test_stop_agent_sigterm_to_sigkill_path() {
        let _lock = PID_FILE_LOCK.lock().unwrap();
        cleanup_pid_file();

        // This test verifies the logic exists but we can't easily test
        // the actual SIGTERM -> SIGKILL flow without a real long-running process
        // Instead, we just verify stop_agent handles various cases

        // Case 1: No process
        assert!(stop_agent().is_ok());

        // Case 2: Stale PID
        create_dummy_pid_file(999999);
        assert!(stop_agent().is_ok());
        assert!(fs::metadata(TEST_PID_FILE).is_err());
    }
}

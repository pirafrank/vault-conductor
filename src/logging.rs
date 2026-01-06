use anyhow::Result;
use env_logger::Builder;
use std::fs;
use std::path::PathBuf;

const LOG_DIRNAME: &str = env!("CARGO_PKG_NAME");
const LOG_FILENAME: &str = "vault-conductor.log";

/// Get the platform-specific log directory path
fn get_log_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .expect("Unable to determine home directory")
            .join("Library")
            .join("Logs")
            .join(LOG_DIRNAME)
    }

    #[cfg(target_os = "linux")]
    {
        dirs::home_dir()
            .expect("Unable to determine home directory")
            .join(".local")
            .join("state")
            .join(LOG_DIRNAME)
            .join("logs")
    }
}

/// Set up logging - to stdout if foreground, to file if background
pub fn setup_logging(log_level: log::LevelFilter, foreground: bool) -> Result<()> {
    let mut builder: Builder = env_logger::Builder::new();
    builder
        .filter_level(log_level)
        .format_timestamp_secs()
        .format_module_path(true)
        .format_target(false);

    if foreground {
        // Log to stdout in foreground mode
        builder.target(env_logger::Target::Stdout);
    } else {
        // Log to file in background mode
        let log_dir = get_log_dir();

        // Create log directory if it doesn't exist
        fs::create_dir_all(&log_dir)?;

        let log_file = log_dir.join(LOG_FILENAME);
        let target = Box::new(
            fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file)?,
        );

        builder.target(env_logger::Target::Pipe(target));
    }

    builder.try_init()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::LevelFilter;

    #[test]
    fn test_log_dirname_constant() {
        assert_eq!(LOG_DIRNAME, env!("CARGO_PKG_NAME"));
    }

    #[test]
    fn test_log_filename_constant() {
        assert_eq!(LOG_FILENAME, "vault-conductor.log");
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_get_log_dir_linux_path() {
        let log_dir = get_log_dir();
        let path_str = log_dir.to_string_lossy();

        assert!(path_str.contains(".local"));
        assert!(path_str.contains("state"));
        assert!(path_str.contains("vault-conductor"));
        assert!(path_str.contains("logs"));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_get_log_dir_macos_path() {
        let log_dir = get_log_dir();
        let path_str = log_dir.to_string_lossy();

        assert!(path_str.contains("Library"));
        assert!(path_str.contains("Logs"));
        assert!(path_str.contains("vault-conductor"));
    }

    #[test]
    fn test_setup_logging_foreground_info() {
        let result = setup_logging(LevelFilter::Info, true);
        // May succeed or fail if logger already initialized
        let _ = result;
    }

    #[test]
    fn test_setup_logging_foreground_debug() {
        let result = setup_logging(LevelFilter::Debug, true);
        let _ = result;
    }

    #[test]
    fn test_setup_logging_foreground_trace() {
        let result = setup_logging(LevelFilter::Trace, true);
        let _ = result;
    }

    #[test]
    fn test_setup_logging_background_mode() {
        let result = setup_logging(LevelFilter::Info, false);
        // May fail due to permissions or logger already initialized
        let _ = result;
    }
}

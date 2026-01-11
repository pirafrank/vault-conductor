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

/// Get the full log file path
pub fn get_log_file_path() -> PathBuf {
    get_log_dir().join(LOG_FILENAME)
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

    builder.init();

    Ok(())
}

use anyhow::Result;
use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use log::{debug, info};

mod bitwarden;
mod config;
use crate::bitwarden::client_wrapper::start_agent;

/// A Rust CLI boilerplate application
#[derive(Parser)]
#[command(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    long_about = None
)]
struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    command: Commands,

    /// Control verbosity level (use -v, -vv, -vvv, or -vvvv for more verbose output)
    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,
}

/// Available subcommands
#[derive(Subcommand)]
enum Commands {
    /// Start the SSH Agent
    StartAgent,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up logging based on verbosity flags
    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .format_timestamp(None)
        .format_module_path(false)
        .format_target(false)
        .init();

    debug!("*** Debug logging enabled ***");
    info!("Starting application");

    // Handle subcommands
    match cli.command {
        Commands::StartAgent => {
            start_agent().await?;
        }
    }

    Ok(())
}

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use log::{debug, info};

mod bitwarden;
mod config;
mod logging;
mod process_manager;
use crate::bitwarden::client_wrapper::start_agent_foreground;
use crate::logging::setup_logging;
use crate::process_manager::{restart_agent, start_agent_background, stop_agent};

#[derive(Parser, Clone)]
struct StartArgs {
    /// Start the agent in foreground
    #[arg(long = "fg", default_value = "false")]
    start_in_foreground: bool,
}

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
    /// Start the SSH Agent in the background
    #[command(name = "start-agent")]
    Start(StartArgs),
    /// Stop the background SSH Agent
    #[command(name = "stop-agent")]
    Stop,
    /// Restart the background SSH Agent
    #[command(name = "restart-agent")]
    Restart,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up logging to file based on verbosity flags
    setup_logging(cli.verbose.log_level_filter())?;

    debug!("*** Debug logging enabled ***");
    info!("Starting application");

    // Handle subcommands
    match cli.command {
        Commands::Start(args) => {
            if args.start_in_foreground {
                start_agent_foreground()
                    .await
                    .context("Failed to start agent in foreground")?;
            } else {
                start_agent_background().context("Failed to start agent")?;
            }
        }
        Commands::Stop => {
            stop_agent().context("Failed to stop agent")?;
        }
        Commands::Restart => {
            restart_agent().await.context("Failed to restart agent")?;
        }
    }

    Ok(())
}

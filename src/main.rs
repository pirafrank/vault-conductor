use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use log::{debug, info};

mod bitwarden;
mod config;
mod file_manager;
mod logging;
mod process_manager;
use crate::bitwarden::client_wrapper::start_agent_foreground;
use crate::logging::setup_logging;
use crate::process_manager::{start_agent_background, stop_agent};

#[derive(Parser, Clone)]
struct StartArgs {
    /// Start the agent in foreground
    #[arg(long = "fg", default_value = "false")]
    start_in_foreground: bool,

    /// Path to the configuration file
    #[arg(long = "config", required = false)]
    config_file: Option<String>,
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
    Start(StartArgs),
    /// Stop the background SSH Agent
    Stop,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Determine if we're running in foreground mode
    // note: using 'ref' to avoid consuming the args and have to clone it
    let foreground = matches!(cli.command, Commands::Start(ref args) if args.start_in_foreground);
    let is_child = std::env::var("VC_DAEMON_CHILD").is_ok();
    let log_to_stdout = foreground && !is_child;

    // Set up logging to stdout if foreground, to file if background
    setup_logging(cli.verbose.log_level_filter(), log_to_stdout)?;

    debug!("*** Debug logging enabled ***");
    info!("Starting application");

    // Handle subcommands
    match cli.command {
        Commands::Start(args) => {
            if args.start_in_foreground {
                start_agent_foreground(args.config_file)
                    .await
                    .context("Failed to start agent in foreground")?;
            } else {
                start_agent_background(args.config_file).context("Failed to start agent")?;
            }
        }
        Commands::Stop => {
            stop_agent().context("Failed to stop agent")?;
        }
    }

    Ok(())
}

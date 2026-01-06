use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use log::{debug, info};

use vault_conductor::bitwarden::client_wrapper::start_agent_foreground;
use vault_conductor::logging::setup_logging;
use vault_conductor::process_manager::{restart_agent, start_agent_background, stop_agent};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_args_default() {
        // Test that we can create StartArgs
        let args = StartArgs {
            start_in_foreground: false,
        };
        assert!(!args.start_in_foreground);

        let args_fg = StartArgs {
            start_in_foreground: true,
        };
        assert!(args_fg.start_in_foreground);
    }

    #[test]
    fn test_start_args_clone() {
        let args = StartArgs {
            start_in_foreground: true,
        };
        let cloned = args.clone();
        assert_eq!(args.start_in_foreground, cloned.start_in_foreground);
    }

    #[test]
    fn test_commands_variants() {
        // Test that Commands enum variants can be constructed
        let start_cmd = Commands::Start(StartArgs {
            start_in_foreground: false,
        });
        assert!(matches!(start_cmd, Commands::Start(_)));

        let stop_cmd = Commands::Stop;
        assert!(matches!(stop_cmd, Commands::Stop));

        let restart_cmd = Commands::Restart;
        assert!(matches!(restart_cmd, Commands::Restart));
    }

    #[test]
    fn test_cli_structure() {
        // Test that CLI structure is well-formed
        // This is mostly a compile-time test
        use clap::CommandFactory;
        let _cmd = Cli::command();
        // If this compiles and runs, the CLI structure is valid
    }

    #[test]
    fn test_verbosity_levels() {
        use clap::CommandFactory;
        let mut cmd = Cli::command();

        // Verify the CLI has verbosity flags
        let help = cmd.render_help().to_string();
        assert!(help.contains("verbose") || help.contains("VERBOSE"));
    }
}

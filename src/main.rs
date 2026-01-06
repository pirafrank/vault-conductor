use anyhow::Result;
use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use log::{debug, info};

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
    /// Example command that greets someone
    Greet {
        /// Name of the person to greet
        #[arg(default_value = "World")]
        name: String,
    },

    /// Example command that echoes input
    Echo {
        /// Message to echo
        message: String,
    },
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    // Set up logging based on verbosity flags
    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .format_timestamp(None)
        .format_module_path(false)
        .format_target(false)
        .init();

    debug!("Debug logging enabled");
    info!("Starting application");

    // Handle subcommands
    match cli.command {
        Commands::Greet { name } => {
            info!("Executing greet command");
            println!("Hello, {}!", name);
        }
        Commands::Echo { message } => {
            info!("Executing echo command");
            println!("{}", message);
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let result = run();

    if let Err(ref e) = result {
        log::error!("Application error: {:?}", e);
    }

    result
}

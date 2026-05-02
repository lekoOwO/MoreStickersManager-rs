#![doc = "Command-line client for MoreStickersManager-rs."]

pub mod client;
pub mod command;
pub mod error;
pub mod output;

pub use error::{CliError, CliResult};

/// Runs the CLI using process arguments and exits with a printable error on failure.
///
/// # Errors
///
/// Returns an error when argument parsing, file I/O, JSON handling, or HTTP requests fail.
pub async fn run_from_env() -> CliResult<()> {
    command::run_from_env().await
}

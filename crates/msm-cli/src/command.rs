use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

use crate::CliResult;

#[derive(Clone, Debug, Parser, PartialEq, Eq)]
#[command(name = "msm", about = "MoreStickersManager-rs CLI")]
pub struct Cli {
    #[arg(long, default_value = "http://127.0.0.1:8080")]
    pub base_url: String,

    #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
    pub output_format: OutputFormat,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Debug, Subcommand, PartialEq, Eq)]
pub enum Command {
    Health,
    Packs {
        #[command(subcommand)]
        command: PackCommand,
    },
}

#[derive(Clone, Debug, Subcommand, PartialEq, Eq)]
pub enum PackCommand {
    List {
        #[arg(long)]
        user_id: String,
    },
    Import {
        #[arg(long)]
        tenant_id: String,
        #[arg(long)]
        owner_user_id: String,
        #[arg(long)]
        pack_id: String,
        #[arg(long, value_enum)]
        visibility: PackVisibility,
        #[arg(long)]
        file: PathBuf,
    },
    Export {
        #[arg(long)]
        pack_id: String,
        #[arg(long)]
        output: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Human,
    Json,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PackVisibility {
    Public,
    Private,
}

impl PackVisibility {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
        }
    }
}

/// Runs the CLI from process arguments.
///
/// # Errors
///
/// Returns an error when command execution fails.
pub async fn run_from_env() -> CliResult<()> {
    let _cli = Cli::parse();
    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::command::{Cli, Command, OutputFormat, PackCommand, PackVisibility};

    #[test]
    fn parses_health_command() {
        let cli = Cli::parse_from(["msm", "health"]);

        assert_eq!(cli.output_format, OutputFormat::Human);
        assert!(matches!(cli.command, Command::Health));
    }

    #[test]
    fn parses_pack_list_command() {
        let cli = Cli::parse_from(["msm", "packs", "list", "--user-id", "user_1"]);

        assert!(matches!(
            cli.command,
            Command::Packs {
                command: PackCommand::List { ref user_id }
            } if user_id == "user_1"
        ));
    }

    #[test]
    fn parses_pack_import_command() {
        let cli = Cli::parse_from([
            "msm",
            "packs",
            "import",
            "--tenant-id",
            "tenant_1",
            "--owner-user-id",
            "user_1",
            "--pack-id",
            "pack_1",
            "--visibility",
            "private",
            "--file",
            "pack.stickerpack",
        ]);

        assert!(matches!(
            cli.command,
            Command::Packs {
                command: PackCommand::Import {
                    visibility: PackVisibility::Private,
                    ..
                }
            }
        ));
    }

    #[test]
    fn parses_pack_export_command_with_json_output() {
        let cli = Cli::parse_from([
            "msm",
            "--output-format",
            "json",
            "packs",
            "export",
            "--pack-id",
            "pack_1",
            "--output",
            "-",
        ]);

        assert_eq!(cli.output_format, OutputFormat::Json);
        assert!(matches!(
            cli.command,
            Command::Packs {
                command: PackCommand::Export {
                    ref pack_id,
                    ref output,
                }
            } if pack_id == "pack_1" && output == "-"
        ));
    }
}

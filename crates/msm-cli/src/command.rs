use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use msm_domain::StickerPack;

use crate::{
    client::{ImportPackPayload, MsmClient, ReqwestMsmClient},
    output::{format_export, format_health, format_import, format_pack_list},
    CliError, CliResult,
};

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

/// Executes a parsed CLI command with the given MSM client.
///
/// # Errors
///
/// Returns an error when client calls, file reads, file writes, or JSON handling fail.
pub async fn execute_with_client<C: MsmClient + Sync>(cli: Cli, client: &C) -> CliResult<String> {
    match cli.command {
        Command::Health => format_health(cli.output_format, &client.health().await?),
        Command::Packs { command } => match command {
            PackCommand::List { user_id } => {
                let packs = client.list_packs(&user_id).await?;
                format_pack_list(cli.output_format, &packs)
            }
            PackCommand::Import {
                tenant_id,
                owner_user_id,
                pack_id,
                visibility,
                file,
            } => {
                let content = read_file(&file)?;
                let pack = StickerPack::from_json_str(&content)?;
                client
                    .import_pack(ImportPackPayload {
                        tenant_id,
                        owner_user_id,
                        pack_id: pack_id.clone(),
                        visibility,
                        pack,
                    })
                    .await?;
                format_import(cli.output_format, &pack_id)
            }
            PackCommand::Export { pack_id, output } => {
                let pack = client.export_pack(&pack_id).await?;
                let json = format_export(&pack)?;
                if output == "-" {
                    Ok(json)
                } else {
                    write_file(&PathBuf::from(&output), &json)?;
                    Ok(format!("exported {pack_id} to {output}"))
                }
            }
        },
    }
}

/// Runs the CLI from process arguments.
///
/// # Errors
///
/// Returns an error when command execution fails.
pub async fn run_from_env() -> CliResult<()> {
    let cli = Cli::parse();
    let client = ReqwestMsmClient::new(&cli.base_url)?;
    let output = execute_with_client(cli, &client).await?;
    if !output.is_empty() {
        println!("{output}");
    }
    Ok(())
}

fn read_file(path: &PathBuf) -> CliResult<String> {
    std::fs::read_to_string(path).map_err(|source| CliError::Io {
        path: path.clone(),
        source,
    })
}

fn write_file(path: &PathBuf, content: &str) -> CliResult<()> {
    std::fs::write(path, content).map_err(|source| CliError::Io {
        path: path.clone(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use clap::Parser;
    use msm_domain::Sticker;
    use std::sync::Mutex;

    use crate::{
        client::{ImportPackPayload, MsmClient},
        command::{execute_with_client, Cli, Command, OutputFormat, PackCommand, PackVisibility},
        output::HealthResponse,
        CliResult,
    };

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

    #[tokio::test]
    async fn executes_health_command() {
        let output =
            execute_with_client(Cli::parse_from(["msm", "health"]), &FakeClient::default())
                .await
                .unwrap();

        assert_eq!(output, "ok");
    }

    #[tokio::test]
    async fn executes_pack_list_command() {
        let output = execute_with_client(
            Cli::parse_from(["msm", "packs", "list", "--user-id", "user_1"]),
            &FakeClient::default(),
        )
        .await
        .unwrap();

        assert_eq!(output, "MoreStickers:Telegram:Pack:sample\tSample");
    }

    #[tokio::test]
    async fn executes_pack_import_command() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("pack.stickerpack");
        std::fs::write(&file, serde_json::to_string(&sample_pack()).unwrap()).unwrap();
        let client = FakeClient::default();

        let output = execute_with_client(
            Cli::parse_from([
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
                file.to_str().unwrap(),
            ]),
            &client,
        )
        .await
        .unwrap();

        assert_eq!(output, "imported pack_1");
        assert_eq!(
            client.imported.lock().unwrap().as_ref().unwrap().pack_id,
            "pack_1"
        );
    }

    #[tokio::test]
    async fn executes_pack_export_to_stdout() {
        let output = execute_with_client(
            Cli::parse_from([
                "msm",
                "packs",
                "export",
                "--pack-id",
                "pack_1",
                "--output",
                "-",
            ]),
            &FakeClient::default(),
        )
        .await
        .unwrap();
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(json["id"], "MoreStickers:Telegram:Pack:sample");
    }

    #[tokio::test]
    async fn import_missing_file_returns_error() {
        let error = execute_with_client(
            Cli::parse_from([
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
                "missing.stickerpack",
            ]),
            &FakeClient::default(),
        )
        .await
        .unwrap_err();

        assert!(error.to_string().contains("missing.stickerpack"));
    }

    #[derive(Default)]
    struct FakeClient {
        imported: Mutex<Option<ImportPackPayload>>,
    }

    #[async_trait]
    impl MsmClient for FakeClient {
        async fn health(&self) -> CliResult<HealthResponse> {
            Ok(HealthResponse {
                status: "ok".to_owned(),
            })
        }

        async fn list_packs(&self, _user_id: &str) -> CliResult<Vec<msm_domain::StickerPack>> {
            Ok(vec![sample_pack()])
        }

        async fn import_pack(&self, payload: ImportPackPayload) -> CliResult<()> {
            *self.imported.lock().unwrap() = Some(payload);
            Ok(())
        }

        async fn export_pack(&self, _pack_id: &str) -> CliResult<msm_domain::StickerPack> {
            Ok(sample_pack())
        }
    }

    fn sample_pack() -> msm_domain::StickerPack {
        let sticker = Sticker {
            id: "MoreStickers:Telegram:Sticker:sample:file".to_owned(),
            image: "https://msm.example/assets/packs/sample/file.webp".to_owned(),
            title: "file".to_owned(),
            sticker_pack_id: "MoreStickers:Telegram:Pack:sample".to_owned(),
            filename: Some("file.webp".to_owned()),
            is_animated: Some(false),
        };

        msm_domain::StickerPack {
            id: "MoreStickers:Telegram:Pack:sample".to_owned(),
            title: "Sample".to_owned(),
            author: None,
            logo: sticker.clone(),
            stickers: vec![sticker],
        }
    }
}

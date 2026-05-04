use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use msm_domain::StickerPack;

use crate::{
    client::{
        CreatePersonalAccessTokenPayload, ImportPackPayload, MsmClient, ReqwestMsmClient,
        UpdatePackPayload,
    },
    output::{
        format_export, format_health, format_import, format_pack_delete, format_pack_list,
        format_pack_rename, format_pat_create, format_pat_list, format_pat_revoke,
    },
    CliError, CliResult,
};

#[derive(Clone, Debug, Parser, PartialEq, Eq)]
#[command(name = "msm", about = "MoreStickersManager-rs CLI")]
pub struct Cli {
    #[arg(long, default_value = "http://127.0.0.1:8080")]
    pub base_url: String,

    #[arg(long)]
    pub pat: Option<String>,

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
    Pats {
        #[command(subcommand)]
        command: PatCommand,
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
    Rename {
        #[arg(long)]
        pack_id: String,
        #[arg(long)]
        title: String,
        #[arg(long, value_enum)]
        visibility: PackVisibility,
    },
    Delete {
        #[arg(long)]
        pack_id: String,
    },
}

#[derive(Clone, Debug, Subcommand, PartialEq, Eq)]
pub enum PatCommand {
    Create {
        #[arg(long)]
        id: String,
        #[arg(long)]
        user_id: String,
        #[arg(long)]
        name: String,
        #[arg(long = "scope")]
        scopes: Vec<String>,
        #[arg(long)]
        expires_at: Option<String>,
    },
    List {
        #[arg(long)]
        user_id: String,
    },
    Revoke {
        #[arg(long)]
        token_id: String,
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
            PackCommand::Rename {
                pack_id,
                title,
                visibility,
            } => {
                client
                    .update_pack(UpdatePackPayload {
                        pack_id: pack_id.clone(),
                        title,
                        visibility,
                    })
                    .await?;
                format_pack_rename(cli.output_format, &pack_id)
            }
            PackCommand::Delete { pack_id } => {
                client.delete_pack(&pack_id).await?;
                format_pack_delete(cli.output_format, &pack_id)
            }
        },
        Command::Pats { command } => match command {
            PatCommand::Create {
                id,
                user_id,
                name,
                scopes,
                expires_at,
            } => {
                let response = client
                    .create_pat(CreatePersonalAccessTokenPayload {
                        id,
                        user_id,
                        name,
                        scopes,
                        expires_at,
                    })
                    .await?;
                format_pat_create(cli.output_format, &response)
            }
            PatCommand::List { user_id } => {
                let tokens = client.list_pats(&user_id).await?;
                format_pat_list(cli.output_format, &tokens)
            }
            PatCommand::Revoke { token_id } => {
                client.revoke_pat(&token_id).await?;
                format_pat_revoke(cli.output_format, &token_id)
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
    let pat = cli.pat.clone().or_else(|| {
        std::env::var("MSM_PAT")
            .ok()
            .filter(|token| !token.is_empty())
    });
    let client = ReqwestMsmClient::new_with_pat(&cli.base_url, pat)?;
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
        client::{
            CreatePersonalAccessTokenPayload, CreatedPersonalAccessToken, ImportPackPayload,
            MsmClient, PersonalAccessToken,
        },
        command::{
            execute_with_client, Cli, Command, OutputFormat, PackCommand, PackVisibility,
            PatCommand,
        },
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
    fn parses_global_pat_option() {
        let cli = Cli::parse_from(["msm", "--pat", "msm_pat_cli1_secret", "health"]);

        assert_eq!(cli.pat.as_deref(), Some("msm_pat_cli1_secret"));
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

    #[test]
    fn parses_pack_rename_command() {
        let cli = Cli::parse_from([
            "msm",
            "packs",
            "rename",
            "--pack-id",
            "pack_1",
            "--title",
            "Renamed Pack",
            "--visibility",
            "public",
        ]);

        assert!(matches!(
            cli.command,
            Command::Packs {
                command: PackCommand::Rename {
                    ref pack_id,
                    ref title,
                    visibility: PackVisibility::Public,
                }
            } if pack_id == "pack_1" && title == "Renamed Pack"
        ));
    }

    #[test]
    fn parses_pack_delete_command() {
        let cli = Cli::parse_from(["msm", "packs", "delete", "--pack-id", "pack_1"]);

        assert!(matches!(
            cli.command,
            Command::Packs {
                command: PackCommand::Delete { ref pack_id }
            } if pack_id == "pack_1"
        ));
    }

    #[test]
    fn parses_pats_create_command() {
        let cli = Cli::parse_from([
            "msm",
            "pats",
            "create",
            "--id",
            "cli1",
            "--user-id",
            "user_1",
            "--name",
            "CLI",
            "--scope",
            "pack.read",
            "--scope",
            "asset.read",
            "--expires-at",
            "2026-05-05T00:00:00Z",
        ]);

        assert!(matches!(
            cli.command,
            Command::Pats {
                command: PatCommand::Create {
                    ref id,
                    ref user_id,
                    ref name,
                    ref scopes,
                    ref expires_at,
                }
            } if id == "cli1"
                && user_id == "user_1"
                && name == "CLI"
                && scopes == &["pack.read".to_owned(), "asset.read".to_owned()]
                && expires_at.as_deref() == Some("2026-05-05T00:00:00Z")
        ));
    }

    #[test]
    fn parses_pats_list_command() {
        let cli = Cli::parse_from(["msm", "pats", "list", "--user-id", "user_1"]);

        assert!(matches!(
            cli.command,
            Command::Pats {
                command: PatCommand::List { ref user_id }
            } if user_id == "user_1"
        ));
    }

    #[test]
    fn parses_pats_revoke_command() {
        let cli = Cli::parse_from(["msm", "pats", "revoke", "--token-id", "cli1"]);

        assert!(matches!(
            cli.command,
            Command::Pats {
                command: PatCommand::Revoke { ref token_id }
            } if token_id == "cli1"
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
    async fn executes_pack_rename_command() {
        let client = FakeClient::default();
        let output = execute_with_client(
            Cli::parse_from([
                "msm",
                "packs",
                "rename",
                "--pack-id",
                "pack_1",
                "--title",
                "Renamed Pack",
                "--visibility",
                "public",
            ]),
            &client,
        )
        .await
        .unwrap();

        assert_eq!(output, "renamed pack_1");
        let renamed = client.renamed_pack.lock().unwrap();
        let renamed = renamed.as_ref().unwrap();
        assert_eq!(renamed.pack_id, "pack_1");
        assert_eq!(renamed.title, "Renamed Pack");
        assert_eq!(renamed.visibility, PackVisibility::Public);
    }

    #[tokio::test]
    async fn executes_pack_delete_command() {
        let client = FakeClient::default();
        let output = execute_with_client(
            Cli::parse_from(["msm", "packs", "delete", "--pack-id", "pack_1"]),
            &client,
        )
        .await
        .unwrap();

        assert_eq!(output, "deleted pack_1");
        assert_eq!(
            client.deleted_pack.lock().unwrap().as_deref(),
            Some("pack_1")
        );
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

    #[tokio::test]
    async fn executes_pats_create_command() {
        let client = FakeClient::default();
        let output = execute_with_client(
            Cli::parse_from([
                "msm",
                "pats",
                "create",
                "--id",
                "cli1",
                "--user-id",
                "user_1",
                "--name",
                "CLI",
                "--scope",
                "pack.read",
                "--scope",
                "asset.read",
            ]),
            &client,
        )
        .await
        .unwrap();

        assert_eq!(output, "created cli1\nmsm_pat_cli1_secret");
        assert_eq!(
            client.created_pat.lock().unwrap().as_ref().unwrap().scopes,
            vec!["pack.read".to_owned(), "asset.read".to_owned()]
        );
    }

    #[tokio::test]
    async fn executes_pats_list_command() {
        let output = execute_with_client(
            Cli::parse_from(["msm", "pats", "list", "--user-id", "user_1"]),
            &FakeClient::default(),
        )
        .await
        .unwrap();

        assert_eq!(output, "cli1\tCLI\tpack.read,asset.read");
    }

    #[tokio::test]
    async fn executes_pats_revoke_command() {
        let client = FakeClient::default();
        let output = execute_with_client(
            Cli::parse_from(["msm", "pats", "revoke", "--token-id", "cli1"]),
            &client,
        )
        .await
        .unwrap();

        assert_eq!(output, "revoked cli1");
        assert_eq!(client.revoked_pat.lock().unwrap().as_deref(), Some("cli1"));
    }

    #[derive(Default)]
    struct FakeClient {
        imported: Mutex<Option<ImportPackPayload>>,
        renamed_pack: Mutex<Option<crate::client::UpdatePackPayload>>,
        deleted_pack: Mutex<Option<String>>,
        created_pat: Mutex<Option<CreatePersonalAccessTokenPayload>>,
        revoked_pat: Mutex<Option<String>>,
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

        async fn update_pack(&self, payload: crate::client::UpdatePackPayload) -> CliResult<()> {
            *self.renamed_pack.lock().unwrap() = Some(payload);
            Ok(())
        }

        async fn delete_pack(&self, pack_id: &str) -> CliResult<()> {
            *self.deleted_pack.lock().unwrap() = Some(pack_id.to_owned());
            Ok(())
        }

        async fn create_pat(
            &self,
            payload: CreatePersonalAccessTokenPayload,
        ) -> CliResult<CreatedPersonalAccessToken> {
            *self.created_pat.lock().unwrap() = Some(payload);
            Ok(CreatedPersonalAccessToken {
                id: "cli1".to_owned(),
                user_id: "user_1".to_owned(),
                name: "CLI".to_owned(),
                scopes: vec!["pack.read".to_owned(), "asset.read".to_owned()],
                token: "msm_pat_cli1_secret".to_owned(),
                created_at: "2026-05-04T00:00:00Z".to_owned(),
                expires_at: None,
                revoked_at: None,
            })
        }

        async fn list_pats(&self, _user_id: &str) -> CliResult<Vec<PersonalAccessToken>> {
            Ok(vec![PersonalAccessToken {
                id: "cli1".to_owned(),
                user_id: "user_1".to_owned(),
                name: "CLI".to_owned(),
                scopes: vec!["pack.read".to_owned(), "asset.read".to_owned()],
                created_at: "2026-05-04T00:00:00Z".to_owned(),
                expires_at: None,
                revoked_at: None,
            }])
        }

        async fn revoke_pat(&self, token_id: &str) -> CliResult<()> {
            *self.revoked_pat.lock().unwrap() = Some(token_id.to_owned());
            Ok(())
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

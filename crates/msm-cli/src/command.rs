use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use msm_domain::StickerPack;

use crate::{
    client::{
        CreateExportJobPayload, CreateExportTargetPayload, CreateFolderPayload,
        CreatePersonalAccessTokenPayload, CreateProviderImportJobPayload,
        CreateProviderImportPlanPayload, CreateSubscriptionAccessTokenPayload,
        CreateSubscriptionGroupPayload, CreateTagPayload, ImportPackPayload, ImportUserDataPayload,
        MsmClient, ReqwestMsmClient, SubscriptionAccessResourceType, UpdateExportTargetPayload,
        UpdatePackPayload, UpdateTenantSettingsPayload, UpdateTenantUserStatusPayload,
        UpsertOidcProviderPayload, UpsertProviderConfigPayload, UpsertTenantRolePayload,
    },
    output::{
        format_export, format_export_job, format_export_job_events, format_export_target,
        format_export_target_kinds, format_export_target_update, format_export_targets,
        format_folder, format_folder_pack, format_folders, format_health, format_import,
        format_membership_remove, format_oidc_provider, format_oidc_provider_delete,
        format_oidc_providers, format_pack_delete, format_pack_ids, format_pack_list,
        format_pack_rename, format_pack_tag, format_pat_create, format_pat_list, format_pat_revoke,
        format_pat_scope_policy, format_provider_config, format_provider_config_delete,
        format_provider_configs, format_provider_import_job, format_provider_import_job_events,
        format_provider_import_plan, format_subscription_group, format_subscription_group_pack,
        format_subscription_groups, format_subscription_link_revoke,
        format_subscription_link_secret, format_subscription_links, format_tag, format_tag_ids,
        format_tags, format_telegram_publication, format_telegram_publications,
        format_tenant_member, format_tenant_members, format_tenant_role, format_tenant_roles,
        format_tenant_settings, format_tenant_user,
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
    Tenants {
        #[command(subcommand)]
        command: TenantCommand,
    },
    Metadata {
        #[command(subcommand)]
        command: MetadataCommand,
    },
    SubscriptionLinks {
        #[command(subcommand)]
        command: SubscriptionLinkCommand,
    },
    Exports {
        #[command(subcommand)]
        command: ExportCommand,
    },
    Providers {
        #[command(subcommand)]
        command: ProviderCommand,
    },
    Portability {
        #[command(subcommand)]
        command: PortabilityCommand,
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
pub enum PortabilityCommand {
    Export {
        #[arg(long)]
        user_id: String,
        #[arg(long)]
        output: String,
    },
    Import {
        #[arg(long)]
        tenant_id: String,
        #[arg(long)]
        file: PathBuf,
    },
}

#[derive(Clone, Debug, Subcommand, PartialEq, Eq)]
pub enum ProviderCommand {
    Plan {
        #[arg(long)]
        tenant_id: String,
        #[arg(long)]
        owner_user_id: String,
        #[arg(long)]
        provider_id: String,
        #[arg(long)]
        remote_id: String,
        #[arg(long)]
        base_url: Option<String>,
    },
    Jobs {
        #[command(subcommand)]
        command: ProviderImportJobCommand,
    },
    Configs {
        #[command(subcommand)]
        command: ProviderConfigCommand,
    },
}

#[derive(Clone, Debug, Subcommand, PartialEq, Eq)]
pub enum ProviderImportJobCommand {
    Create {
        #[arg(long)]
        id: String,
        #[arg(long)]
        tenant_id: String,
        #[arg(long)]
        owner_user_id: String,
        #[arg(long)]
        provider_id: String,
        #[arg(long)]
        remote_id: String,
        #[arg(long)]
        target_pack_id: Option<String>,
        #[arg(long)]
        base_url: Option<String>,
    },
    Get {
        #[arg(long)]
        job_id: String,
    },
    Events {
        #[arg(long)]
        job_id: String,
    },
}

#[derive(Clone, Debug, Subcommand, PartialEq, Eq)]
pub enum ProviderConfigCommand {
    List {
        #[arg(long)]
        tenant_id: String,
    },
    Upsert {
        #[arg(long)]
        id: String,
        #[arg(long)]
        tenant_id: String,
        #[arg(long)]
        provider_id: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        config_json: String,
        #[arg(long = "disabled", default_value_t = true, action = clap::ArgAction::SetFalse)]
        is_enabled: bool,
    },
    Delete {
        #[arg(long)]
        id: String,
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
    ScopePolicy {
        #[arg(long)]
        user_id: String,
    },
    Revoke {
        #[arg(long)]
        token_id: String,
    },
}

#[derive(Clone, Debug, Subcommand, PartialEq, Eq)]
pub enum TenantCommand {
    Members {
        #[command(subcommand)]
        command: TenantMemberCommand,
    },
    Settings {
        #[command(subcommand)]
        command: TenantSettingsCommand,
    },
    Users {
        #[command(subcommand)]
        command: TenantUserCommand,
    },
    Roles {
        #[command(subcommand)]
        command: TenantRoleCommand,
    },
    OidcProviders {
        #[command(subcommand)]
        command: TenantOidcProviderCommand,
    },
}

#[derive(Clone, Debug, Subcommand, PartialEq, Eq)]
pub enum TenantMemberCommand {
    List {
        #[arg(long)]
        tenant_id: String,
    },
    SetRole {
        #[arg(long)]
        tenant_id: String,
        #[arg(long)]
        user_id: String,
        #[arg(long, value_enum)]
        role: TenantMemberRoleArg,
    },
}

#[derive(Clone, Debug, Subcommand, PartialEq, Eq)]
pub enum TenantSettingsCommand {
    Get {
        #[arg(long)]
        tenant_id: String,
    },
    Update {
        #[arg(long)]
        tenant_id: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        public_asset_url: Option<String>,
        #[arg(long, default_value_t = false)]
        local_registration_enabled: bool,
    },
}

#[derive(Clone, Debug, Subcommand, PartialEq, Eq)]
pub enum TenantUserCommand {
    SetStatus {
        #[arg(long)]
        tenant_id: String,
        #[arg(long)]
        user_id: String,
        #[arg(long)]
        disabled: bool,
    },
}

#[derive(Clone, Debug, Subcommand, PartialEq, Eq)]
pub enum TenantRoleCommand {
    List {
        #[arg(long)]
        tenant_id: String,
    },
    Upsert {
        #[arg(long)]
        tenant_id: String,
        #[arg(long)]
        role_id: String,
        #[arg(long)]
        name: String,
        #[arg(long = "permission")]
        permissions: Vec<String>,
    },
}

#[derive(Clone, Debug, Subcommand, PartialEq, Eq)]
pub enum TenantOidcProviderCommand {
    List {
        #[arg(long)]
        tenant_id: String,
    },
    Upsert {
        #[arg(long)]
        tenant_id: String,
        #[arg(long)]
        provider_id: String,
        #[arg(long)]
        display_name: String,
        #[arg(long)]
        issuer_url: String,
        #[arg(long)]
        client_id: String,
        #[arg(long)]
        client_secret: String,
        #[arg(long = "scope")]
        scopes: Vec<String>,
        #[arg(long = "disabled", default_value_t = true, action = clap::ArgAction::SetFalse)]
        is_enabled: bool,
        #[arg(long = "deny-registration", default_value_t = true, action = clap::ArgAction::SetFalse)]
        allow_registration: bool,
    },
    Delete {
        #[arg(long)]
        tenant_id: String,
        #[arg(long)]
        provider_id: String,
    },
}

#[derive(Clone, Debug, Subcommand, PartialEq, Eq)]
pub enum MetadataCommand {
    Folders {
        #[command(subcommand)]
        command: FolderCommand,
    },
    PackTags {
        #[command(subcommand)]
        command: PackTagCommand,
    },
    Tags {
        #[command(subcommand)]
        command: TagCommand,
    },
    SubscriptionGroups {
        #[command(subcommand)]
        command: SubscriptionGroupCommand,
    },
}

#[derive(Clone, Debug, Subcommand, PartialEq, Eq)]
pub enum FolderCommand {
    List {
        #[arg(long)]
        tenant_id: String,
        #[arg(long)]
        owner_user_id: String,
    },
    Create {
        #[arg(long)]
        id: String,
        #[arg(long)]
        tenant_id: String,
        #[arg(long)]
        owner_user_id: String,
        #[arg(long)]
        name: String,
    },
    Packs {
        #[command(subcommand)]
        command: FolderPackCommand,
    },
}

#[derive(Clone, Debug, Subcommand, PartialEq, Eq)]
pub enum FolderPackCommand {
    List {
        #[arg(long)]
        folder_id: String,
    },
    Add {
        #[arg(long)]
        folder_id: String,
        #[arg(long)]
        pack_id: String,
        #[arg(long, default_value_t = 0)]
        sort_order: i64,
    },
    Remove {
        #[arg(long)]
        folder_id: String,
        #[arg(long)]
        pack_id: String,
    },
}

#[derive(Clone, Debug, Subcommand, PartialEq, Eq)]
pub enum PackTagCommand {
    List {
        #[arg(long)]
        pack_id: String,
    },
    Add {
        #[arg(long)]
        pack_id: String,
        #[arg(long)]
        tag_id: String,
    },
    Remove {
        #[arg(long)]
        pack_id: String,
        #[arg(long)]
        tag_id: String,
    },
}

#[derive(Clone, Debug, Subcommand, PartialEq, Eq)]
pub enum TagCommand {
    List {
        #[arg(long)]
        tenant_id: String,
    },
    Create {
        #[arg(long)]
        id: String,
        #[arg(long)]
        tenant_id: String,
        #[arg(long)]
        name: String,
    },
}

#[derive(Clone, Debug, Subcommand, PartialEq, Eq)]
pub enum SubscriptionGroupCommand {
    List {
        #[arg(long)]
        tenant_id: String,
        #[arg(long)]
        owner_user_id: String,
    },
    Create {
        #[arg(long)]
        id: String,
        #[arg(long)]
        tenant_id: String,
        #[arg(long)]
        owner_user_id: String,
        #[arg(long)]
        title: String,
        #[arg(long, value_enum)]
        visibility: PackVisibility,
    },
    Packs {
        #[command(subcommand)]
        command: SubscriptionGroupPackCommand,
    },
}

#[derive(Clone, Debug, Subcommand, PartialEq, Eq)]
pub enum SubscriptionGroupPackCommand {
    List {
        #[arg(long)]
        subscription_group_id: String,
    },
    Add {
        #[arg(long)]
        subscription_group_id: String,
        #[arg(long)]
        pack_id: String,
        #[arg(long, default_value_t = 0)]
        sort_order: i64,
    },
    Remove {
        #[arg(long)]
        subscription_group_id: String,
        #[arg(long)]
        pack_id: String,
    },
}

#[derive(Clone, Debug, Subcommand, PartialEq, Eq)]
pub enum SubscriptionLinkCommand {
    Create {
        #[arg(long)]
        id: String,
        #[arg(long, value_enum)]
        resource_type: SubscriptionLinkResourceType,
        #[arg(long)]
        resource_id: String,
    },
    List {
        #[arg(long)]
        user_id: String,
    },
    Rotate {
        #[arg(long)]
        token_id: String,
    },
    Revoke {
        #[arg(long)]
        token_id: String,
    },
}

#[derive(Clone, Debug, Subcommand, PartialEq, Eq)]
pub enum ExportCommand {
    Kinds,
    Targets {
        #[command(subcommand)]
        command: ExportTargetCommand,
    },
    Jobs {
        #[command(subcommand)]
        command: ExportJobCommand,
    },
    Publications {
        #[command(subcommand)]
        command: ExportPublicationCommand,
    },
}

#[derive(Clone, Debug, Subcommand, PartialEq, Eq)]
pub enum ExportTargetCommand {
    List {
        #[arg(long)]
        tenant_id: String,
    },
    Create {
        #[arg(long)]
        id: String,
        #[arg(long)]
        tenant_id: String,
        #[arg(long)]
        kind: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        config_json: String,
        #[arg(long)]
        disabled: bool,
    },
    Update {
        #[arg(long)]
        target_id: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        config_json: String,
        #[arg(long)]
        disabled: bool,
    },
    Delete {
        #[arg(long)]
        target_id: String,
    },
}

#[derive(Clone, Debug, Subcommand, PartialEq, Eq)]
pub enum ExportJobCommand {
    Create {
        #[arg(long)]
        id: String,
        #[arg(long)]
        tenant_id: String,
        #[arg(long)]
        source_pack_id: String,
        #[arg(long)]
        target_id: String,
        #[arg(long, default_value = "{}")]
        options_json: String,
        #[arg(long)]
        telegram_set_name_slug: Option<String>,
        #[arg(long)]
        telegram_default_emoji: Option<String>,
        #[arg(long, conflicts_with = "telegram_live")]
        telegram_dry_run: bool,
        #[arg(long, conflicts_with = "telegram_dry_run")]
        telegram_live: bool,
        #[arg(long, value_enum)]
        telegram_reconcile_mode: Option<TelegramReconcileModeArg>,
        #[arg(long)]
        execute_reconciliation: bool,
        #[arg(long)]
        allow_destructive_reconciliation: bool,
    },
    Get {
        #[arg(long)]
        job_id: String,
    },
    Requeue {
        #[arg(long)]
        job_id: String,
    },
    Events {
        #[arg(long)]
        job_id: String,
    },
}

#[derive(Clone, Debug, Subcommand, PartialEq, Eq)]
pub enum ExportPublicationCommand {
    List {
        #[arg(long)]
        pack_id: String,
    },
    Get {
        #[arg(long)]
        publication_id: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Human,
    Json,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum TelegramReconcileModeArg {
    CreateOnly,
    AppendMissing,
    Mirror,
}

impl TelegramReconcileModeArg {
    const fn as_worker_value(self) -> &'static str {
        match self {
            Self::CreateOnly => "createOnly",
            Self::AppendMissing => "appendMissing",
            Self::Mirror => "mirror",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum TenantMemberRoleArg {
    Admin,
    User,
}

impl TenantMemberRoleArg {
    const fn as_api_value(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::User => "user",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PackVisibility {
    Public,
    Private,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum SubscriptionLinkResourceType {
    Pack,
    SubscriptionGroup,
}

impl From<SubscriptionLinkResourceType> for SubscriptionAccessResourceType {
    fn from(value: SubscriptionLinkResourceType) -> Self {
        match value {
            SubscriptionLinkResourceType::Pack => Self::Pack,
            SubscriptionLinkResourceType::SubscriptionGroup => Self::SubscriptionGroup,
        }
    }
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
#[allow(clippy::too_many_lines)]
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
        Command::Portability { command } => match command {
            PortabilityCommand::Export { user_id, output } => {
                let export = client.export_user_data(&user_id).await?;
                let json = serde_json::to_string_pretty(&export)?;
                if output == "-" {
                    Ok(json)
                } else {
                    write_file(&PathBuf::from(&output), &json)?;
                    Ok(format!(
                        "exported portable user data for {user_id} to {output}"
                    ))
                }
            }
            PortabilityCommand::Import { tenant_id, file } => {
                let content = read_file(&file)?;
                let export = serde_json::from_str(&content)?;
                client
                    .import_user_data(ImportUserDataPayload {
                        tenant_id: tenant_id.clone(),
                        export,
                    })
                    .await?;
                Ok(format!("imported portable user data into {tenant_id}"))
            }
        },
        Command::Providers { command } => match command {
            ProviderCommand::Plan {
                tenant_id,
                owner_user_id,
                provider_id,
                remote_id,
                base_url,
            } => {
                let plan = client
                    .create_provider_import_plan(CreateProviderImportPlanPayload {
                        tenant_id,
                        owner_user_id,
                        provider_id,
                        remote_id,
                        base_url,
                    })
                    .await?;
                format_provider_import_plan(cli.output_format, &plan)
            }
            ProviderCommand::Jobs { command } => match command {
                ProviderImportJobCommand::Create {
                    id,
                    tenant_id,
                    owner_user_id,
                    provider_id,
                    remote_id,
                    target_pack_id,
                    base_url,
                } => {
                    let job = client
                        .create_provider_import_job(CreateProviderImportJobPayload {
                            id,
                            tenant_id,
                            owner_user_id,
                            provider_id,
                            remote_id,
                            target_pack_id,
                            base_url,
                        })
                        .await?;
                    format_provider_import_job(cli.output_format, &job)
                }
                ProviderImportJobCommand::Get { job_id } => {
                    let job = client.get_provider_import_job(&job_id).await?;
                    format_provider_import_job(cli.output_format, &job)
                }
                ProviderImportJobCommand::Events { job_id } => {
                    let events = client.list_provider_import_job_events(&job_id).await?;
                    format_provider_import_job_events(cli.output_format, &events)
                }
            },
            ProviderCommand::Configs { command } => match command {
                ProviderConfigCommand::List { tenant_id } => {
                    let configs = client.list_provider_configs(&tenant_id).await?;
                    format_provider_configs(cli.output_format, &configs)
                }
                ProviderConfigCommand::Upsert {
                    id,
                    tenant_id,
                    provider_id,
                    name,
                    config_json,
                    is_enabled,
                } => {
                    let config = serde_json::from_str(&config_json)?;
                    let provider_config = client
                        .upsert_provider_config(
                            &id,
                            UpsertProviderConfigPayload {
                                tenant_id,
                                provider_id,
                                name,
                                config,
                                is_enabled,
                            },
                        )
                        .await?;
                    format_provider_config(cli.output_format, &provider_config)
                }
                ProviderConfigCommand::Delete { id } => {
                    client.delete_provider_config(&id).await?;
                    format_provider_config_delete(cli.output_format, &id)
                }
            },
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
            PatCommand::ScopePolicy { user_id } => {
                let policy = client.get_pat_scope_policy(&user_id).await?;
                format_pat_scope_policy(cli.output_format, &policy)
            }
            PatCommand::Revoke { token_id } => {
                client.revoke_pat(&token_id).await?;
                format_pat_revoke(cli.output_format, &token_id)
            }
        },
        Command::Tenants { command } => match command {
            TenantCommand::Members { command } => match command {
                TenantMemberCommand::List { tenant_id } => {
                    let members = client.list_tenant_members(&tenant_id).await?;
                    format_tenant_members(cli.output_format, &members)
                }
                TenantMemberCommand::SetRole {
                    tenant_id,
                    user_id,
                    role,
                } => {
                    let member = client
                        .set_tenant_member_role(&tenant_id, &user_id, role.as_api_value())
                        .await?;
                    format_tenant_member(cli.output_format, &member)
                }
            },
            TenantCommand::Settings { command } => match command {
                TenantSettingsCommand::Get { tenant_id } => {
                    let settings = client.get_tenant_settings(&tenant_id).await?;
                    format_tenant_settings(cli.output_format, &settings)
                }
                TenantSettingsCommand::Update {
                    tenant_id,
                    name,
                    public_asset_url,
                    local_registration_enabled,
                } => {
                    let settings = client
                        .update_tenant_settings(
                            &tenant_id,
                            UpdateTenantSettingsPayload {
                                name,
                                public_asset_url,
                                local_registration_enabled,
                            },
                        )
                        .await?;
                    format_tenant_settings(cli.output_format, &settings)
                }
            },
            TenantCommand::Users { command } => match command {
                TenantUserCommand::SetStatus {
                    tenant_id,
                    user_id,
                    disabled,
                } => {
                    let user = client
                        .update_tenant_user_status(
                            &tenant_id,
                            &user_id,
                            UpdateTenantUserStatusPayload {
                                is_disabled: disabled,
                            },
                        )
                        .await?;
                    format_tenant_user(cli.output_format, &user)
                }
            },
            TenantCommand::Roles { command } => match command {
                TenantRoleCommand::List { tenant_id } => {
                    let roles = client.list_tenant_roles(&tenant_id).await?;
                    format_tenant_roles(cli.output_format, &roles)
                }
                TenantRoleCommand::Upsert {
                    tenant_id,
                    role_id,
                    name,
                    permissions,
                } => {
                    let role = client
                        .upsert_tenant_role(
                            &tenant_id,
                            &role_id,
                            UpsertTenantRolePayload { name, permissions },
                        )
                        .await?;
                    format_tenant_role(cli.output_format, &role)
                }
            },
            TenantCommand::OidcProviders { command } => match command {
                TenantOidcProviderCommand::List { tenant_id } => {
                    let providers = client.list_oidc_providers(&tenant_id).await?;
                    format_oidc_providers(cli.output_format, &providers)
                }
                TenantOidcProviderCommand::Upsert {
                    tenant_id,
                    provider_id,
                    display_name,
                    issuer_url,
                    client_id,
                    client_secret,
                    scopes,
                    is_enabled,
                    allow_registration,
                } => {
                    let provider = client
                        .upsert_oidc_provider(
                            &tenant_id,
                            &provider_id,
                            UpsertOidcProviderPayload {
                                display_name,
                                issuer_url,
                                client_id,
                                client_secret,
                                scopes,
                                is_enabled,
                                allow_registration,
                            },
                        )
                        .await?;
                    format_oidc_provider(cli.output_format, &provider)
                }
                TenantOidcProviderCommand::Delete {
                    tenant_id,
                    provider_id,
                } => {
                    client
                        .delete_oidc_provider(&tenant_id, &provider_id)
                        .await?;
                    format_oidc_provider_delete(cli.output_format, &tenant_id, &provider_id)
                }
            },
        },
        Command::Metadata { command } => match command {
            MetadataCommand::Folders { command } => match command {
                FolderCommand::List {
                    tenant_id,
                    owner_user_id,
                } => {
                    let folders = client.list_folders(&tenant_id, &owner_user_id).await?;
                    format_folders(cli.output_format, &folders)
                }
                FolderCommand::Create {
                    id,
                    tenant_id,
                    owner_user_id,
                    name,
                } => {
                    let folder = client
                        .create_folder(CreateFolderPayload {
                            id,
                            tenant_id,
                            owner_user_id,
                            name,
                        })
                        .await?;
                    format_folder(cli.output_format, &folder)
                }
                FolderCommand::Packs { command } => match command {
                    FolderPackCommand::List { folder_id } => {
                        let pack_ids = client.list_folder_pack_ids(&folder_id).await?;
                        format_pack_ids(cli.output_format, &pack_ids)
                    }
                    FolderPackCommand::Add {
                        folder_id,
                        pack_id,
                        sort_order,
                    } => {
                        let link = client
                            .add_pack_to_folder(&folder_id, &pack_id, sort_order)
                            .await?;
                        format_folder_pack(cli.output_format, &link)
                    }
                    FolderPackCommand::Remove { folder_id, pack_id } => {
                        client.remove_pack_from_folder(&folder_id, &pack_id).await?;
                        format_membership_remove(cli.output_format, &folder_id, &pack_id)
                    }
                },
            },
            MetadataCommand::PackTags { command } => match command {
                PackTagCommand::List { pack_id } => {
                    let tag_ids = client.list_pack_tag_ids(&pack_id).await?;
                    format_tag_ids(cli.output_format, &tag_ids)
                }
                PackTagCommand::Add { pack_id, tag_id } => {
                    let link = client.add_tag_to_pack(&pack_id, &tag_id).await?;
                    format_pack_tag(cli.output_format, &link)
                }
                PackTagCommand::Remove { pack_id, tag_id } => {
                    client.remove_tag_from_pack(&pack_id, &tag_id).await?;
                    format_membership_remove(cli.output_format, &pack_id, &tag_id)
                }
            },
            MetadataCommand::Tags { command } => match command {
                TagCommand::List { tenant_id } => {
                    let tags = client.list_tags(&tenant_id).await?;
                    format_tags(cli.output_format, &tags)
                }
                TagCommand::Create {
                    id,
                    tenant_id,
                    name,
                } => {
                    let tag = client
                        .create_tag(CreateTagPayload {
                            id,
                            tenant_id,
                            name,
                        })
                        .await?;
                    format_tag(cli.output_format, &tag)
                }
            },
            MetadataCommand::SubscriptionGroups { command } => match command {
                SubscriptionGroupCommand::List {
                    tenant_id,
                    owner_user_id,
                } => {
                    let groups = client
                        .list_subscription_groups(&tenant_id, &owner_user_id)
                        .await?;
                    format_subscription_groups(cli.output_format, &groups)
                }
                SubscriptionGroupCommand::Create {
                    id,
                    tenant_id,
                    owner_user_id,
                    title,
                    visibility,
                } => {
                    let group = client
                        .create_subscription_group(CreateSubscriptionGroupPayload {
                            id,
                            tenant_id,
                            owner_user_id,
                            title,
                            visibility,
                        })
                        .await?;
                    format_subscription_group(cli.output_format, &group)
                }
                SubscriptionGroupCommand::Packs { command } => match command {
                    SubscriptionGroupPackCommand::List {
                        subscription_group_id,
                    } => {
                        let pack_ids = client
                            .list_subscription_group_pack_ids(&subscription_group_id)
                            .await?;
                        format_pack_ids(cli.output_format, &pack_ids)
                    }
                    SubscriptionGroupPackCommand::Add {
                        subscription_group_id,
                        pack_id,
                        sort_order,
                    } => {
                        let link = client
                            .add_pack_to_subscription_group(
                                &subscription_group_id,
                                &pack_id,
                                sort_order,
                            )
                            .await?;
                        format_subscription_group_pack(cli.output_format, &link)
                    }
                    SubscriptionGroupPackCommand::Remove {
                        subscription_group_id,
                        pack_id,
                    } => {
                        client
                            .remove_pack_from_subscription_group(&subscription_group_id, &pack_id)
                            .await?;
                        format_membership_remove(
                            cli.output_format,
                            &subscription_group_id,
                            &pack_id,
                        )
                    }
                },
            },
        },
        Command::SubscriptionLinks { command } => match command {
            SubscriptionLinkCommand::Create {
                id,
                resource_type,
                resource_id,
            } => {
                let link = client
                    .create_subscription_access_token(CreateSubscriptionAccessTokenPayload {
                        id,
                        resource_type: resource_type.into(),
                        resource_id,
                    })
                    .await?;
                format_subscription_link_secret(cli.output_format, &link)
            }
            SubscriptionLinkCommand::List { user_id } => {
                let links = client.list_subscription_access_tokens(&user_id).await?;
                format_subscription_links(cli.output_format, &links)
            }
            SubscriptionLinkCommand::Rotate { token_id } => {
                let link = client.rotate_subscription_access_token(&token_id).await?;
                format_subscription_link_secret(cli.output_format, &link)
            }
            SubscriptionLinkCommand::Revoke { token_id } => {
                client.revoke_subscription_access_token(&token_id).await?;
                format_subscription_link_revoke(cli.output_format, &token_id)
            }
        },
        Command::Exports { command } => match command {
            ExportCommand::Kinds => {
                let kinds = client.list_export_target_kinds().await?;
                format_export_target_kinds(cli.output_format, &kinds)
            }
            ExportCommand::Targets { command } => match command {
                ExportTargetCommand::List { tenant_id } => {
                    let targets = client.list_export_targets(&tenant_id).await?;
                    format_export_targets(cli.output_format, &targets)
                }
                ExportTargetCommand::Create {
                    id,
                    tenant_id,
                    kind,
                    name,
                    config_json,
                    disabled,
                } => {
                    let config = serde_json::from_str(&config_json)?;
                    let target = client
                        .create_export_target(CreateExportTargetPayload {
                            id,
                            tenant_id,
                            kind,
                            name,
                            config,
                            is_enabled: !disabled,
                        })
                        .await?;
                    format_export_target(cli.output_format, &target)
                }
                ExportTargetCommand::Update {
                    target_id,
                    name,
                    config_json,
                    disabled,
                } => {
                    let config: serde_json::Value = serde_json::from_str(&config_json)?;
                    let target = client
                        .update_export_target(UpdateExportTargetPayload {
                            target_id,
                            name,
                            config,
                            is_enabled: !disabled,
                        })
                        .await?;
                    format_export_target_update(cli.output_format, &target)
                }
                ExportTargetCommand::Delete { target_id } => {
                    client.delete_export_target(&target_id).await?;
                    Ok(format!("deleted {target_id}"))
                }
            },
            ExportCommand::Jobs { command } => match command {
                ExportJobCommand::Create {
                    id,
                    tenant_id,
                    source_pack_id,
                    target_id,
                    options_json,
                    telegram_set_name_slug,
                    telegram_default_emoji,
                    telegram_dry_run,
                    telegram_live,
                    telegram_reconcile_mode,
                    execute_reconciliation,
                    allow_destructive_reconciliation,
                } => {
                    let options = build_export_job_options(
                        &options_json,
                        TelegramExportOptionOverrides {
                            telegram_set_name_slug,
                            telegram_default_emoji,
                            dry_run: if telegram_dry_run {
                                Some(TelegramDryRunOverride::DryRun)
                            } else if telegram_live {
                                Some(TelegramDryRunOverride::Live)
                            } else {
                                None
                            },
                            telegram_reconcile_mode,
                            execute_reconciliation: execute_reconciliation
                                .then_some(ReconciliationExecutionFlag),
                            allow_destructive_reconciliation: allow_destructive_reconciliation
                                .then_some(DestructiveReconciliationFlag),
                        },
                    )?;
                    let job = client
                        .create_export_job(CreateExportJobPayload {
                            id,
                            tenant_id,
                            source_pack_id,
                            target_id,
                            options,
                        })
                        .await?;
                    format_export_job(cli.output_format, &job)
                }
                ExportJobCommand::Get { job_id } => {
                    let job = client.get_export_job(&job_id).await?;
                    format_export_job(cli.output_format, &job)
                }
                ExportJobCommand::Requeue { job_id } => {
                    let job = client.requeue_export_job(&job_id).await?;
                    format_export_job(cli.output_format, &job)
                }
                ExportJobCommand::Events { job_id } => {
                    let events = client.list_export_job_events(&job_id).await?;
                    format_export_job_events(cli.output_format, &events)
                }
            },
            ExportCommand::Publications { command } => match command {
                ExportPublicationCommand::List { pack_id } => {
                    let publications = client.list_telegram_publications(&pack_id).await?;
                    format_telegram_publications(cli.output_format, &publications)
                }
                ExportPublicationCommand::Get { publication_id } => {
                    let publication = client.get_telegram_publication(&publication_id).await?;
                    format_telegram_publication(cli.output_format, &publication)
                }
            },
        },
    }
}

struct TelegramExportOptionOverrides {
    telegram_set_name_slug: Option<String>,
    telegram_default_emoji: Option<String>,
    dry_run: Option<TelegramDryRunOverride>,
    telegram_reconcile_mode: Option<TelegramReconcileModeArg>,
    execute_reconciliation: Option<ReconciliationExecutionFlag>,
    allow_destructive_reconciliation: Option<DestructiveReconciliationFlag>,
}

enum TelegramDryRunOverride {
    DryRun,
    Live,
}

struct ReconciliationExecutionFlag;

struct DestructiveReconciliationFlag;

impl TelegramDryRunOverride {
    const fn as_worker_value(&self) -> bool {
        match self {
            Self::DryRun => true,
            Self::Live => false,
        }
    }
}

fn build_export_job_options(
    options_json: &str,
    overrides: TelegramExportOptionOverrides,
) -> CliResult<serde_json::Value> {
    let mut options: serde_json::Value = serde_json::from_str(options_json)?;
    if options.is_null() {
        options = serde_json::json!({});
    }
    let Some(object) = options.as_object_mut() else {
        return Err(CliError::Client(
            "export job options JSON must be an object".to_owned(),
        ));
    };

    if let Some(value) = overrides.telegram_set_name_slug {
        object.insert("setNameSlug".to_owned(), serde_json::Value::String(value));
    }
    if let Some(value) = overrides.telegram_default_emoji {
        object.insert("defaultEmoji".to_owned(), serde_json::Value::String(value));
    }
    if let Some(value) = overrides.dry_run {
        object.insert(
            "dryRun".to_owned(),
            serde_json::Value::Bool(value.as_worker_value()),
        );
    }
    if let Some(value) = overrides.telegram_reconcile_mode {
        object.insert(
            "reconcileMode".to_owned(),
            serde_json::Value::String(value.as_worker_value().to_owned()),
        );
    }
    if overrides.execute_reconciliation.is_some() {
        object.insert(
            "executeReconciliation".to_owned(),
            serde_json::Value::Bool(true),
        );
    }
    if overrides.allow_destructive_reconciliation.is_some() {
        object.insert(
            "allowDestructiveReconciliation".to_owned(),
            serde_json::Value::Bool(true),
        );
    }

    Ok(options)
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
            CreateExportJobPayload, CreateExportTargetPayload, CreateFolderPayload,
            CreatePersonalAccessTokenPayload, CreateProviderImportJobPayload,
            CreateProviderImportPlanPayload, CreateSubscriptionAccessTokenPayload,
            CreateSubscriptionGroupPayload, CreateTagPayload, CreatedPersonalAccessToken,
            CreatedSubscriptionAccessToken, ExportJob, ExportJobEvent, ExportTarget,
            ExportTargetKind, Folder, FolderPack, ImportPackPayload, ImportUserDataPayload,
            MsmClient, OidcProvider, PackTag, PatScopePolicy, PersonalAccessToken, ProviderConfig,
            ProviderHttpRequestPlan, ProviderImportJob, ProviderImportJobEvent, ProviderImportPlan,
            SubscriptionAccessResourceType, SubscriptionAccessToken, SubscriptionGroup,
            SubscriptionGroupPack, Tag, TelegramPublication, TenantMember, TenantRole,
            TenantSettings, TenantUser, UpdateExportTargetPayload, UpdateTenantSettingsPayload,
            UpdateTenantUserStatusPayload, UpsertOidcProviderPayload, UpsertProviderConfigPayload,
            UpsertTenantRolePayload,
        },
        command::{
            execute_with_client, Cli, Command, ExportCommand, ExportJobCommand,
            ExportPublicationCommand, ExportTargetCommand, FolderCommand, FolderPackCommand,
            MetadataCommand, OutputFormat, PackCommand, PackTagCommand, PackVisibility, PatCommand,
            ProviderCommand, ProviderConfigCommand, ProviderImportJobCommand,
            SubscriptionGroupCommand, SubscriptionGroupPackCommand, SubscriptionLinkCommand,
            SubscriptionLinkResourceType, TagCommand, TenantCommand, TenantMemberCommand,
            TenantMemberRoleArg, TenantOidcProviderCommand, TenantRoleCommand,
            TenantSettingsCommand, TenantUserCommand,
        },
        output::HealthResponse,
        CliResult,
    };

    type TenantRoleUpsertCall = (String, String, String, Vec<String>);
    type OidcProviderUpsertCall = (String, String, UpsertOidcProviderPayload);
    type ProviderConfigUpsertCall = (String, UpsertProviderConfigPayload);

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
    fn parses_provider_import_plan_command() {
        let cli = Cli::parse_from([
            "msm",
            "providers",
            "plan",
            "--tenant-id",
            "tenant_1",
            "--owner-user-id",
            "user_1",
            "--provider-id",
            "line-stickers",
            "--remote-id",
            "line_cats",
            "--base-url",
            "https://store.line.me",
        ]);

        assert!(matches!(
            cli.command,
            Command::Providers {
                command: ProviderCommand::Plan {
                    ref tenant_id,
                    ref owner_user_id,
                    ref provider_id,
                    ref remote_id,
                    ref base_url,
                }
            } if tenant_id == "tenant_1"
                && owner_user_id == "user_1"
                && provider_id == "line-stickers"
                && remote_id == "line_cats"
                && base_url.as_deref() == Some("https://store.line.me")
        ));
    }

    #[test]
    fn parses_provider_import_job_commands() {
        let create = Cli::parse_from([
            "msm",
            "providers",
            "jobs",
            "create",
            "--id",
            "provider_job_1",
            "--tenant-id",
            "tenant_1",
            "--owner-user-id",
            "user_1",
            "--provider-id",
            "line-stickers",
            "--remote-id",
            "line_cats",
            "--target-pack-id",
            "pack_line_cats",
        ]);
        assert!(matches!(
            create.command,
            Command::Providers {
                command: ProviderCommand::Jobs {
                    command: ProviderImportJobCommand::Create {
                        ref id,
                        ref tenant_id,
                        ref owner_user_id,
                        ref provider_id,
                        ref remote_id,
                        ref target_pack_id,
                        ..
                    }
                }
            } if id == "provider_job_1"
                && tenant_id == "tenant_1"
                && owner_user_id == "user_1"
                && provider_id == "line-stickers"
                && remote_id == "line_cats"
                && target_pack_id.as_deref() == Some("pack_line_cats")
        ));

        let get = Cli::parse_from([
            "msm",
            "providers",
            "jobs",
            "get",
            "--job-id",
            "provider_job_1",
        ]);
        assert!(matches!(
            get.command,
            Command::Providers {
                command: ProviderCommand::Jobs {
                    command: ProviderImportJobCommand::Get { ref job_id }
                }
            } if job_id == "provider_job_1"
        ));

        let events = Cli::parse_from([
            "msm",
            "providers",
            "jobs",
            "events",
            "--job-id",
            "provider_job_1",
        ]);
        assert!(matches!(
            events.command,
            Command::Providers {
                command: ProviderCommand::Jobs {
                    command: ProviderImportJobCommand::Events { ref job_id }
                }
            } if job_id == "provider_job_1"
        ));
    }

    #[test]
    fn parses_provider_config_commands() {
        let list = Cli::parse_from([
            "msm",
            "providers",
            "configs",
            "list",
            "--tenant-id",
            "tenant_1",
        ]);
        assert!(matches!(
            list.command,
            Command::Providers {
                command: ProviderCommand::Configs {
                    command: ProviderConfigCommand::List { ref tenant_id }
                }
            } if tenant_id == "tenant_1"
        ));

        let upsert = Cli::parse_from([
            "msm",
            "providers",
            "configs",
            "upsert",
            "--id",
            "provider_telegram",
            "--tenant-id",
            "tenant_1",
            "--provider-id",
            "telegram",
            "--name",
            "Telegram Import Bot",
            "--config-json",
            r#"{"botToken":"123456:secret"}"#,
            "--disabled",
        ]);
        assert!(matches!(
            upsert.command,
            Command::Providers {
                command: ProviderCommand::Configs {
                    command: ProviderConfigCommand::Upsert {
                        ref id,
                        ref tenant_id,
                        ref provider_id,
                        ref name,
                        ref config_json,
                        is_enabled: false,
                    }
                }
            } if id == "provider_telegram"
                && tenant_id == "tenant_1"
                && provider_id == "telegram"
                && name == "Telegram Import Bot"
                && config_json == r#"{"botToken":"123456:secret"}"#
        ));

        let delete = Cli::parse_from([
            "msm",
            "providers",
            "configs",
            "delete",
            "--id",
            "provider_telegram",
        ]);
        assert!(matches!(
            delete.command,
            Command::Providers {
                command: ProviderCommand::Configs {
                    command: ProviderConfigCommand::Delete { ref id }
                }
            } if id == "provider_telegram"
        ));
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
    fn parses_pats_scope_policy_command() {
        let cli = Cli::parse_from(["msm", "pats", "scope-policy", "--user-id", "user_1"]);

        assert!(matches!(
            cli.command,
            Command::Pats {
                command: PatCommand::ScopePolicy { ref user_id }
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

    #[test]
    fn parses_subscription_link_commands() {
        let create = Cli::parse_from([
            "msm",
            "subscription-links",
            "create",
            "--id",
            "packlink",
            "--resource-type",
            "pack",
            "--resource-id",
            "pack_1",
        ]);
        assert!(matches!(
            create.command,
            Command::SubscriptionLinks {
                command: SubscriptionLinkCommand::Create {
                    ref id,
                    resource_type: SubscriptionLinkResourceType::Pack,
                    ref resource_id,
                }
            } if id == "packlink" && resource_id == "pack_1"
        ));

        let list = Cli::parse_from(["msm", "subscription-links", "list", "--user-id", "user_1"]);
        assert!(matches!(
            list.command,
            Command::SubscriptionLinks {
                command: SubscriptionLinkCommand::List { ref user_id }
            } if user_id == "user_1"
        ));

        let rotate = Cli::parse_from([
            "msm",
            "subscription-links",
            "rotate",
            "--token-id",
            "packlink",
        ]);
        assert!(matches!(
            rotate.command,
            Command::SubscriptionLinks {
                command: SubscriptionLinkCommand::Rotate { ref token_id }
            } if token_id == "packlink"
        ));

        let revoke = Cli::parse_from([
            "msm",
            "subscription-links",
            "revoke",
            "--token-id",
            "packlink",
        ]);
        assert!(matches!(
            revoke.command,
            Command::SubscriptionLinks {
                command: SubscriptionLinkCommand::Revoke { ref token_id }
            } if token_id == "packlink"
        ));
    }

    #[test]
    fn parses_tenant_member_commands() {
        let list = Cli::parse_from([
            "msm",
            "tenants",
            "members",
            "list",
            "--tenant-id",
            "tenant_1",
        ]);
        assert!(matches!(
            list.command,
            Command::Tenants {
                command: TenantCommand::Members {
                    command: TenantMemberCommand::List { ref tenant_id }
                }
            } if tenant_id == "tenant_1"
        ));

        let set_role = Cli::parse_from([
            "msm",
            "tenants",
            "members",
            "set-role",
            "--tenant-id",
            "tenant_1",
            "--user-id",
            "user_2",
            "--role",
            "admin",
        ]);
        assert!(matches!(
            set_role.command,
            Command::Tenants {
                command: TenantCommand::Members {
                    command: TenantMemberCommand::SetRole {
                        ref tenant_id,
                        ref user_id,
                        role: TenantMemberRoleArg::Admin,
                    }
                }
            } if tenant_id == "tenant_1" && user_id == "user_2"
        ));
    }

    #[test]
    fn parses_tenant_administration_parity_commands() {
        let settings_get = Cli::parse_from([
            "msm",
            "tenants",
            "settings",
            "get",
            "--tenant-id",
            "tenant_1",
        ]);
        assert!(matches!(
            settings_get.command,
            Command::Tenants {
                command: TenantCommand::Settings {
                    command: TenantSettingsCommand::Get { ref tenant_id }
                }
            } if tenant_id == "tenant_1"
        ));

        let settings_update = Cli::parse_from([
            "msm",
            "tenants",
            "settings",
            "update",
            "--tenant-id",
            "tenant_1",
            "--name",
            "Production",
            "--public-asset-url",
            "https://cdn.example.test/msm",
        ]);
        assert!(matches!(
            settings_update.command,
            Command::Tenants {
                command: TenantCommand::Settings {
                    command: TenantSettingsCommand::Update {
                        ref tenant_id,
                        ref name,
                        ref public_asset_url,
                        local_registration_enabled,
                    }
                }
            } if tenant_id == "tenant_1"
                && name == "Production"
                && public_asset_url.as_deref() == Some("https://cdn.example.test/msm")
                && !local_registration_enabled
        ));

        let user_status = Cli::parse_from([
            "msm",
            "tenants",
            "users",
            "set-status",
            "--tenant-id",
            "tenant_1",
            "--user-id",
            "user_2",
            "--disabled",
        ]);
        assert!(matches!(
            user_status.command,
            Command::Tenants {
                command: TenantCommand::Users {
                    command: TenantUserCommand::SetStatus {
                        ref tenant_id,
                        ref user_id,
                        disabled: true,
                    }
                }
            } if tenant_id == "tenant_1" && user_id == "user_2"
        ));

        let roles_upsert = Cli::parse_from([
            "msm",
            "tenants",
            "roles",
            "upsert",
            "--tenant-id",
            "tenant_1",
            "--role-id",
            "role_editor",
            "--name",
            "Editors",
            "--permission",
            "pack.read",
            "--permission",
            "pack.update",
        ]);
        assert!(matches!(
            roles_upsert.command,
            Command::Tenants {
                command: TenantCommand::Roles {
                    command: TenantRoleCommand::Upsert {
                        ref tenant_id,
                        ref role_id,
                        ref name,
                        ref permissions,
                    }
                }
            } if tenant_id == "tenant_1"
                && role_id == "role_editor"
                && name == "Editors"
                && permissions == &vec!["pack.read".to_owned(), "pack.update".to_owned()]
        ));
    }

    #[test]
    fn parses_tenant_oidc_provider_commands() {
        let oidc_list = Cli::parse_from([
            "msm",
            "tenants",
            "oidc-providers",
            "list",
            "--tenant-id",
            "tenant_1",
        ]);
        assert!(matches!(
            oidc_list.command,
            Command::Tenants {
                command: TenantCommand::OidcProviders {
                    command: TenantOidcProviderCommand::List { ref tenant_id }
                }
            } if tenant_id == "tenant_1"
        ));

        let oidc_upsert = Cli::parse_from([
            "msm",
            "tenants",
            "oidc-providers",
            "upsert",
            "--tenant-id",
            "tenant_1",
            "--provider-id",
            "google",
            "--display-name",
            "Google Workspace",
            "--issuer-url",
            "https://accounts.google.com",
            "--client-id",
            "client_1",
            "--client-secret",
            "secret_1",
            "--scope",
            "openid",
            "--scope",
            "email",
            "--disabled",
            "--deny-registration",
        ]);
        assert!(matches!(
            oidc_upsert.command,
            Command::Tenants {
                command: TenantCommand::OidcProviders {
                    command: TenantOidcProviderCommand::Upsert {
                        ref tenant_id,
                        ref provider_id,
                        ref display_name,
                        ref issuer_url,
                        ref client_id,
                        ref client_secret,
                        ref scopes,
                        is_enabled: false,
                        allow_registration: false,
                    }
                }
            } if tenant_id == "tenant_1"
                && provider_id == "google"
                && display_name == "Google Workspace"
                && issuer_url == "https://accounts.google.com"
                && client_id == "client_1"
                && client_secret == "secret_1"
                && scopes == &vec!["openid".to_owned(), "email".to_owned()]
        ));

        let oidc_delete = Cli::parse_from([
            "msm",
            "tenants",
            "oidc-providers",
            "delete",
            "--tenant-id",
            "tenant_1",
            "--provider-id",
            "google",
        ]);
        assert!(matches!(
            oidc_delete.command,
            Command::Tenants {
                command: TenantCommand::OidcProviders {
                    command: TenantOidcProviderCommand::Delete {
                        ref tenant_id,
                        ref provider_id,
                    }
                }
            } if tenant_id == "tenant_1" && provider_id == "google"
        ));
    }

    #[test]
    fn parses_export_target_create_command() {
        let cli = Cli::parse_from([
            "msm",
            "exports",
            "targets",
            "create",
            "--id",
            "target_telegram",
            "--tenant-id",
            "tenant_1",
            "--kind",
            "telegram",
            "--name",
            "Telegram",
            "--config-json",
            r#"{"botUsername":"msm_bot"}"#,
        ]);

        assert!(matches!(
            cli.command,
            Command::Exports {
                command: ExportCommand::Targets {
                    command: ExportTargetCommand::Create {
                        ref id,
                        ref tenant_id,
                        ref kind,
                        ..
                    }
                }
            } if id == "target_telegram" && tenant_id == "tenant_1" && kind == "telegram"
        ));
    }

    #[test]
    fn parses_export_job_create_command() {
        let cli = Cli::parse_from([
            "msm",
            "exports",
            "jobs",
            "create",
            "--id",
            "job_1",
            "--tenant-id",
            "tenant_1",
            "--source-pack-id",
            "pack_1",
            "--target-id",
            "target_telegram",
            "--options-json",
            r#"{"setNameSlug":"sample"}"#,
        ]);

        assert!(matches!(
            cli.command,
            Command::Exports {
                command: ExportCommand::Jobs {
                    command: ExportJobCommand::Create {
                        ref id,
                        ref target_id,
                        ..
                    }
                }
            } if id == "job_1" && target_id == "target_telegram"
        ));
    }

    #[test]
    fn parses_export_job_create_telegram_reconciliation_flags() {
        let cli = Cli::parse_from([
            "msm",
            "exports",
            "jobs",
            "create",
            "--id",
            "job_1",
            "--tenant-id",
            "tenant_1",
            "--source-pack-id",
            "pack_1",
            "--target-id",
            "target_telegram",
            "--telegram-live",
            "--telegram-reconcile-mode",
            "append-missing",
            "--execute-reconciliation",
            "--telegram-set-name-slug",
            "sample",
            "--telegram-default-emoji",
            "ok",
        ]);

        assert!(matches!(
            cli.command,
            Command::Exports {
                command: ExportCommand::Jobs {
                    command: ExportJobCommand::Create {
                        telegram_live: true,
                        execute_reconciliation: true,
                        ..
                    }
                }
            }
        ));
    }

    #[test]
    fn parses_export_publication_commands() {
        let list = Cli::parse_from([
            "msm",
            "exports",
            "publications",
            "list",
            "--pack-id",
            "pack_1",
        ]);
        assert!(matches!(
            list.command,
            Command::Exports {
                command: ExportCommand::Publications {
                    command: ExportPublicationCommand::List { ref pack_id }
                }
            } if pack_id == "pack_1"
        ));

        let get = Cli::parse_from([
            "msm",
            "exports",
            "publications",
            "get",
            "--publication-id",
            "telegram_pub_1",
        ]);
        assert!(matches!(
            get.command,
            Command::Exports {
                command: ExportCommand::Publications {
                    command: ExportPublicationCommand::Get { ref publication_id }
                }
            } if publication_id == "telegram_pub_1"
        ));
    }

    #[test]
    fn parses_metadata_commands() {
        let folder_create = Cli::parse_from([
            "msm",
            "metadata",
            "folders",
            "create",
            "--id",
            "folder_1",
            "--tenant-id",
            "tenant_1",
            "--owner-user-id",
            "user_1",
            "--name",
            "Favorites",
        ]);
        assert!(matches!(
            folder_create.command,
            Command::Metadata {
                command: MetadataCommand::Folders {
                    command: FolderCommand::Create {
                        ref id,
                        ref name,
                        ..
                    }
                }
            } if id == "folder_1" && name == "Favorites"
        ));

        let tag_list =
            Cli::parse_from(["msm", "metadata", "tags", "list", "--tenant-id", "tenant_1"]);
        assert!(matches!(
            tag_list.command,
            Command::Metadata {
                command: MetadataCommand::Tags {
                    command: TagCommand::List { ref tenant_id }
                }
            } if tenant_id == "tenant_1"
        ));

        let group_create = Cli::parse_from([
            "msm",
            "metadata",
            "subscription-groups",
            "create",
            "--id",
            "sub_1",
            "--tenant-id",
            "tenant_1",
            "--owner-user-id",
            "user_1",
            "--title",
            "Weekly",
            "--visibility",
            "private",
        ]);
        assert!(matches!(
            group_create.command,
            Command::Metadata {
                command: MetadataCommand::SubscriptionGroups {
                    command: SubscriptionGroupCommand::Create {
                        ref id,
                        visibility: PackVisibility::Private,
                        ..
                    }
                }
            } if id == "sub_1"
        ));
    }

    #[test]
    fn parses_metadata_membership_commands() {
        let folder_pack_add = Cli::parse_from([
            "msm",
            "metadata",
            "folders",
            "packs",
            "add",
            "--folder-id",
            "folder_1",
            "--pack-id",
            "pack_1",
            "--sort-order",
            "10",
        ]);
        assert!(matches!(
            folder_pack_add.command,
            Command::Metadata {
                command: MetadataCommand::Folders {
                    command: FolderCommand::Packs {
                        command: FolderPackCommand::Add {
                            ref folder_id,
                            ref pack_id,
                            sort_order,
                        }
                    }
                }
            } if folder_id == "folder_1" && pack_id == "pack_1" && sort_order == 10
        ));

        let pack_tag_list = Cli::parse_from([
            "msm",
            "metadata",
            "pack-tags",
            "list",
            "--pack-id",
            "pack_1",
        ]);
        assert!(matches!(
            pack_tag_list.command,
            Command::Metadata {
                command: MetadataCommand::PackTags {
                    command: PackTagCommand::List { ref pack_id }
                }
            } if pack_id == "pack_1"
        ));

        let subscription_group_pack_remove = Cli::parse_from([
            "msm",
            "metadata",
            "subscription-groups",
            "packs",
            "remove",
            "--subscription-group-id",
            "sub_1",
            "--pack-id",
            "pack_1",
        ]);
        assert!(matches!(
            subscription_group_pack_remove.command,
            Command::Metadata {
                command: MetadataCommand::SubscriptionGroups {
                    command: SubscriptionGroupCommand::Packs {
                        command: SubscriptionGroupPackCommand::Remove {
                            ref subscription_group_id,
                            ref pack_id,
                        }
                    }
                }
            } if subscription_group_id == "sub_1" && pack_id == "pack_1"
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
    async fn executes_provider_import_plan_command() {
        let client = FakeClient::default();

        let output = execute_with_client(
            Cli::parse_from([
                "msm",
                "providers",
                "plan",
                "--tenant-id",
                "tenant_1",
                "--owner-user-id",
                "user_1",
                "--provider-id",
                "line-stickers",
                "--remote-id",
                "line_cats",
            ]),
            &client,
        )
        .await
        .unwrap();

        assert_eq!(
            output,
            "line-stickers\tline_cats\tGET\thttps://store.line.me/stickershop/product/line_cats/en"
        );
        let payload = client.provider_import_plan.lock().unwrap().clone().unwrap();
        assert_eq!(payload.provider_id, "line-stickers");
        assert_eq!(payload.remote_id, "line_cats");
    }

    #[tokio::test]
    async fn executes_provider_import_job_commands() {
        let client = FakeClient::default();

        let created = execute_with_client(
            Cli::parse_from([
                "msm",
                "providers",
                "jobs",
                "create",
                "--id",
                "provider_job_1",
                "--tenant-id",
                "tenant_1",
                "--owner-user-id",
                "user_1",
                "--provider-id",
                "line-stickers",
                "--remote-id",
                "line_cats",
                "--target-pack-id",
                "pack_line_cats",
            ]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(created, "provider_job_1\tline-stickers\tqueued\t0/3");
        let payload = client
            .created_provider_import_job
            .lock()
            .unwrap()
            .clone()
            .unwrap();
        assert_eq!(payload.target_pack_id.as_deref(), Some("pack_line_cats"));

        let fetched = execute_with_client(
            Cli::parse_from([
                "msm",
                "providers",
                "jobs",
                "get",
                "--job-id",
                "provider_job_1",
            ]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(fetched, "provider_job_1\tline-stickers\tqueued\t0/3");

        let events = execute_with_client(
            Cli::parse_from([
                "msm",
                "providers",
                "jobs",
                "events",
                "--job-id",
                "provider_job_1",
            ]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(events, "1\tinfo\tqueued\tprovider import job queued");
    }

    #[tokio::test]
    async fn executes_provider_config_commands() {
        let client = FakeClient::default();

        let listed = execute_with_client(
            Cli::parse_from([
                "msm",
                "providers",
                "configs",
                "list",
                "--tenant-id",
                "tenant_1",
            ]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(
            listed,
            "provider_telegram\ttelegram\tTelegram Import Bot\tenabled"
        );
        assert_eq!(
            client
                .listed_provider_config_tenant
                .lock()
                .unwrap()
                .as_deref(),
            Some("tenant_1")
        );

        let upserted = execute_with_client(
            Cli::parse_from([
                "msm",
                "providers",
                "configs",
                "upsert",
                "--id",
                "provider_telegram",
                "--tenant-id",
                "tenant_1",
                "--provider-id",
                "telegram",
                "--name",
                "Telegram Import Bot",
                "--config-json",
                r#"{"botToken":"123456:secret"}"#,
                "--disabled",
            ]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(
            upserted,
            "provider_telegram\ttelegram\tTelegram Import Bot\tdisabled"
        );
        let upserted_call = client
            .upserted_provider_config
            .lock()
            .unwrap()
            .clone()
            .unwrap();
        assert_eq!(upserted_call.0, "provider_telegram");
        assert_eq!(upserted_call.1.provider_id, "telegram");
        assert_eq!(upserted_call.1.config["botToken"], "123456:secret");
        assert!(!upserted_call.1.is_enabled);

        let deleted = execute_with_client(
            Cli::parse_from([
                "msm",
                "providers",
                "configs",
                "delete",
                "--id",
                "provider_telegram",
            ]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(deleted, "deleted provider_telegram");
        assert_eq!(
            client.deleted_provider_config.lock().unwrap().as_deref(),
            Some("provider_telegram")
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
    async fn executes_pats_scope_policy_command() {
        let output = execute_with_client(
            Cli::parse_from(["msm", "pats", "scope-policy", "--user-id", "user_1"]),
            &FakeClient::default(),
        )
        .await
        .unwrap();

        assert_eq!(output, "user_1\tpack.read,pat.manage");
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

    #[tokio::test]
    async fn executes_subscription_link_commands() {
        let client = FakeClient::default();
        let create = execute_with_client(
            Cli::parse_from([
                "msm",
                "subscription-links",
                "create",
                "--id",
                "packlink",
                "--resource-type",
                "pack",
                "--resource-id",
                "pack_1",
            ]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(create, "packlink\tmsm_sub_packlink_secret");
        let created = client
            .created_subscription_access_token
            .lock()
            .unwrap()
            .clone()
            .unwrap();
        assert_eq!(created.resource_type, SubscriptionAccessResourceType::Pack);
        assert_eq!(created.resource_id, "pack_1");

        let list = execute_with_client(
            Cli::parse_from(["msm", "subscription-links", "list", "--user-id", "user_1"]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(list, "packlink\tPack\tpack_1\tfalse");

        let rotate = execute_with_client(
            Cli::parse_from([
                "msm",
                "subscription-links",
                "rotate",
                "--token-id",
                "packlink",
            ]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(rotate, "packlink\tmsm_sub_packlink_secret");
        assert_eq!(
            client
                .rotated_subscription_access_token
                .lock()
                .unwrap()
                .as_deref(),
            Some("packlink")
        );

        let revoke = execute_with_client(
            Cli::parse_from([
                "msm",
                "subscription-links",
                "revoke",
                "--token-id",
                "packlink",
            ]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(revoke, "revoked packlink");
        assert_eq!(
            client
                .revoked_subscription_access_token
                .lock()
                .unwrap()
                .as_deref(),
            Some("packlink")
        );
    }

    #[tokio::test]
    async fn executes_tenant_member_commands() {
        let client = FakeClient::default();

        let list = execute_with_client(
            Cli::parse_from([
                "msm",
                "tenants",
                "members",
                "list",
                "--tenant-id",
                "tenant_1",
            ]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(list, "user_1\tadmin\nuser_2\tuser");

        let set_role = execute_with_client(
            Cli::parse_from([
                "msm",
                "tenants",
                "members",
                "set-role",
                "--tenant-id",
                "tenant_1",
                "--user-id",
                "user_2",
                "--role",
                "admin",
            ]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(set_role, "user_2\tadmin");
        assert_eq!(
            client
                .upserted_tenant_member
                .lock()
                .unwrap()
                .as_ref()
                .unwrap(),
            &(
                "tenant_1".to_owned(),
                "user_2".to_owned(),
                "admin".to_owned()
            )
        );
    }

    #[tokio::test]
    async fn executes_tenant_settings_commands() {
        let client = FakeClient::default();
        let settings = execute_with_client(
            Cli::parse_from([
                "msm",
                "tenants",
                "settings",
                "get",
                "--tenant-id",
                "tenant_1",
            ]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(
            settings,
            "tenant_1\tTenant\thttps://cdn.example.test/msm\ttrue"
        );

        let settings_update = execute_with_client(
            Cli::parse_from([
                "msm",
                "tenants",
                "settings",
                "update",
                "--tenant-id",
                "tenant_1",
                "--name",
                "Production",
                "--public-asset-url",
                "https://cdn.example.test/prod",
            ]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(
            settings_update,
            "tenant_1\tProduction\thttps://cdn.example.test/prod\tfalse"
        );
        assert_eq!(
            client
                .updated_tenant_settings
                .lock()
                .unwrap()
                .as_ref()
                .unwrap(),
            &(
                "tenant_1".to_owned(),
                "Production".to_owned(),
                Some("https://cdn.example.test/prod".to_owned()),
                false
            )
        );
    }

    #[tokio::test]
    async fn executes_tenant_user_status_command() {
        let client = FakeClient::default();
        let user_status = execute_with_client(
            Cli::parse_from([
                "msm",
                "tenants",
                "users",
                "set-status",
                "--tenant-id",
                "tenant_1",
                "--user-id",
                "user_2",
                "--disabled",
            ]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(user_status, "user_2\tuser@example.com\tdisabled");
        assert_eq!(
            client
                .updated_tenant_user_status
                .lock()
                .unwrap()
                .as_ref()
                .unwrap(),
            &("tenant_1".to_owned(), "user_2".to_owned(), true)
        );
    }

    #[tokio::test]
    async fn executes_tenant_role_commands() {
        let client = FakeClient::default();
        let roles = execute_with_client(
            Cli::parse_from(["msm", "tenants", "roles", "list", "--tenant-id", "tenant_1"]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(roles, "role_editor\tEditors\tpack.read,pack.update");

        let role = execute_with_client(
            Cli::parse_from([
                "msm",
                "tenants",
                "roles",
                "upsert",
                "--tenant-id",
                "tenant_1",
                "--role-id",
                "role_editor",
                "--name",
                "Editors",
                "--permission",
                "pack.read",
            ]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(role, "role_editor\tEditors\tpack.read");
        assert_eq!(
            client
                .upserted_tenant_role
                .lock()
                .unwrap()
                .as_ref()
                .unwrap(),
            &(
                "tenant_1".to_owned(),
                "role_editor".to_owned(),
                "Editors".to_owned(),
                vec!["pack.read".to_owned()]
            )
        );
    }

    #[tokio::test]
    async fn executes_tenant_oidc_provider_commands() {
        let client = FakeClient::default();

        let providers = execute_with_client(
            Cli::parse_from([
                "msm",
                "tenants",
                "oidc-providers",
                "list",
                "--tenant-id",
                "tenant_1",
            ]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(
            providers,
            "google\tGoogle Workspace\thttps://accounts.google.com\tenabled\tregistration"
        );

        let provider = execute_with_client(
            Cli::parse_from([
                "msm",
                "tenants",
                "oidc-providers",
                "upsert",
                "--tenant-id",
                "tenant_1",
                "--provider-id",
                "google",
                "--display-name",
                "Google Workspace",
                "--issuer-url",
                "https://accounts.google.com",
                "--client-id",
                "client_1",
                "--client-secret",
                "secret_1",
                "--scope",
                "openid",
                "--disabled",
            ]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(
            provider,
            "google\tGoogle Workspace\thttps://accounts.google.com\tdisabled\tregistration"
        );
        let oidc_call = client
            .upserted_oidc_provider
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone();
        assert_eq!(oidc_call.0, "tenant_1");
        assert_eq!(oidc_call.1, "google");
        assert_eq!(oidc_call.2.client_secret, "secret_1");
        assert_eq!(oidc_call.2.scopes, vec!["openid".to_owned()]);
        assert!(!oidc_call.2.is_enabled);
        assert!(oidc_call.2.allow_registration);

        let deleted = execute_with_client(
            Cli::parse_from([
                "msm",
                "tenants",
                "oidc-providers",
                "delete",
                "--tenant-id",
                "tenant_1",
                "--provider-id",
                "google",
            ]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(deleted, "deleted google");
        assert_eq!(
            client
                .deleted_oidc_provider
                .lock()
                .unwrap()
                .as_ref()
                .unwrap(),
            &("tenant_1".to_owned(), "google".to_owned())
        );
    }

    #[tokio::test]
    async fn executes_export_kinds_command() {
        let output = execute_with_client(
            Cli::parse_from(["msm", "exports", "kinds"]),
            &FakeClient::default(),
        )
        .await
        .unwrap();

        assert_eq!(output, "telegram\tTelegram");
    }

    #[tokio::test]
    async fn executes_export_target_create_command() {
        let client = FakeClient::default();
        let output = execute_with_client(
            Cli::parse_from([
                "msm",
                "exports",
                "targets",
                "create",
                "--id",
                "target_telegram",
                "--tenant-id",
                "tenant_1",
                "--kind",
                "telegram",
                "--name",
                "Telegram",
                "--config-json",
                r#"{"botUsername":"msm_bot"}"#,
            ]),
            &client,
        )
        .await
        .unwrap();

        assert_eq!(output, "created target_telegram");
        assert_eq!(
            client
                .created_export_target
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .config["botUsername"],
            "msm_bot"
        );
    }

    #[tokio::test]
    async fn executes_export_target_update_delete_commands() {
        let update_output = execute_with_client(
            Cli::parse_from([
                "msm",
                "exports",
                "targets",
                "update",
                "--target-id",
                "target_telegram",
                "--name",
                "Telegram Updated",
                "--config-json",
                r#"{"botUsername":"msm_bot"}"#,
            ]),
            &FakeClient::default(),
        )
        .await
        .unwrap();
        let delete_output = execute_with_client(
            Cli::parse_from([
                "msm",
                "exports",
                "targets",
                "delete",
                "--target-id",
                "target_telegram",
            ]),
            &FakeClient::default(),
        )
        .await
        .unwrap();

        assert_eq!(update_output, "updated target_telegram");
        assert_eq!(delete_output, "deleted target_telegram");
    }

    #[tokio::test]
    async fn executes_export_job_create_command_with_json_output() {
        let output = execute_with_client(
            Cli::parse_from([
                "msm",
                "--output-format",
                "json",
                "exports",
                "jobs",
                "create",
                "--id",
                "job_1",
                "--tenant-id",
                "tenant_1",
                "--source-pack-id",
                "pack_1",
                "--target-id",
                "target_telegram",
            ]),
            &FakeClient::default(),
        )
        .await
        .unwrap();
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(json["id"], "job_1");
        assert_eq!(json["status"], "queued");
        assert_eq!(json["attemptCount"], 0);
        assert_eq!(json["maxAttempts"], 3);
    }

    #[tokio::test]
    async fn executes_export_job_create_with_telegram_reconciliation_flags() {
        let client = FakeClient::default();
        let output = execute_with_client(
            Cli::parse_from([
                "msm",
                "exports",
                "jobs",
                "create",
                "--id",
                "job_1",
                "--tenant-id",
                "tenant_1",
                "--source-pack-id",
                "pack_1",
                "--target-id",
                "target_telegram",
                "--telegram-live",
                "--telegram-reconcile-mode",
                "append-missing",
                "--execute-reconciliation",
                "--telegram-set-name-slug",
                "sample",
                "--telegram-default-emoji",
                "ok",
            ]),
            &client,
        )
        .await
        .unwrap();

        assert_eq!(output, "job_1\tqueued\t0/3");
        let payload = client.created_export_job.lock().unwrap().clone().unwrap();
        assert_eq!(payload.options["dryRun"], false);
        assert_eq!(payload.options["reconcileMode"], "appendMissing");
        assert_eq!(payload.options["executeReconciliation"], true);
        assert_eq!(payload.options["setNameSlug"], "sample");
        assert_eq!(payload.options["defaultEmoji"], "ok");
    }

    #[tokio::test]
    async fn executes_export_job_events_command() {
        let output = execute_with_client(
            Cli::parse_from(["msm", "exports", "jobs", "events", "--job-id", "job_1"]),
            &FakeClient::default(),
        )
        .await
        .unwrap();

        assert_eq!(output, "1\tinfo\tqueued\tjob queued");
    }

    #[tokio::test]
    async fn executes_export_job_requeue_command() {
        let output = execute_with_client(
            Cli::parse_from(["msm", "exports", "jobs", "requeue", "--job-id", "job_1"]),
            &FakeClient::default(),
        )
        .await
        .unwrap();

        assert_eq!(output, "job_1\tqueued\t0/3");
    }

    #[tokio::test]
    async fn executes_export_publication_commands() {
        let list_output = execute_with_client(
            Cli::parse_from([
                "msm",
                "exports",
                "publications",
                "list",
                "--pack-id",
                "pack_1",
            ]),
            &FakeClient::default(),
        )
        .await
        .unwrap();
        assert_eq!(
            list_output,
            "telegram_pub_1\tsample_by_msm_bot\thttps://t.me/addstickers/sample_by_msm_bot"
        );

        let get_output = execute_with_client(
            Cli::parse_from([
                "msm",
                "exports",
                "publications",
                "get",
                "--publication-id",
                "telegram_pub_1",
            ]),
            &FakeClient::default(),
        )
        .await
        .unwrap();
        assert_eq!(
            get_output,
            "telegram_pub_1\tsample_by_msm_bot\thttps://t.me/addstickers/sample_by_msm_bot"
        );
    }

    #[tokio::test]
    async fn executes_metadata_commands() {
        let client = FakeClient::default();

        let folder_output = execute_with_client(
            Cli::parse_from([
                "msm",
                "metadata",
                "folders",
                "create",
                "--id",
                "folder_1",
                "--tenant-id",
                "tenant_1",
                "--owner-user-id",
                "user_1",
                "--name",
                "Favorites",
            ]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(folder_output, "folder_1\tFavorites");
        assert_eq!(
            client.created_folder.lock().unwrap().as_ref().unwrap().name,
            "Favorites"
        );

        let tags_output = execute_with_client(
            Cli::parse_from(["msm", "metadata", "tags", "list", "--tenant-id", "tenant_1"]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(tags_output, "tag_1\tcute");

        let group_output = execute_with_client(
            Cli::parse_from([
                "msm",
                "metadata",
                "subscription-groups",
                "create",
                "--id",
                "sub_1",
                "--tenant-id",
                "tenant_1",
                "--owner-user-id",
                "user_1",
                "--title",
                "Weekly",
                "--visibility",
                "private",
            ]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(group_output, "sub_1\tWeekly\tprivate");
        assert_eq!(
            client
                .created_subscription_group
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .title,
            "Weekly"
        );
    }

    #[tokio::test]
    async fn executes_metadata_folder_membership_commands() {
        let client = FakeClient::default();

        let add_output = execute_with_client(
            Cli::parse_from([
                "msm",
                "metadata",
                "folders",
                "packs",
                "add",
                "--folder-id",
                "folder_1",
                "--pack-id",
                "pack_1",
                "--sort-order",
                "10",
            ]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(add_output, "folder_1\tpack_1\t10");
        assert_eq!(
            client.added_folder_pack.lock().unwrap().as_ref().unwrap(),
            &("folder_1".to_owned(), "pack_1".to_owned(), 10)
        );

        let list_output = execute_with_client(
            Cli::parse_from([
                "msm",
                "metadata",
                "folders",
                "packs",
                "list",
                "--folder-id",
                "folder_1",
            ]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(list_output, "pack_1");
    }

    #[tokio::test]
    async fn executes_metadata_pack_tag_membership_commands() {
        let client = FakeClient::default();

        let pack_tag_output = execute_with_client(
            Cli::parse_from([
                "msm",
                "metadata",
                "pack-tags",
                "add",
                "--pack-id",
                "pack_1",
                "--tag-id",
                "tag_1",
            ]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(pack_tag_output, "pack_1\ttag_1");

        let pack_tag_remove = execute_with_client(
            Cli::parse_from([
                "msm",
                "metadata",
                "pack-tags",
                "remove",
                "--pack-id",
                "pack_1",
                "--tag-id",
                "tag_1",
            ]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(pack_tag_remove, "removed pack_1 tag_1");
    }

    #[tokio::test]
    async fn executes_metadata_subscription_group_membership_commands() {
        let client = FakeClient::default();

        let subscription_pack_output = execute_with_client(
            Cli::parse_from([
                "msm",
                "metadata",
                "subscription-groups",
                "packs",
                "add",
                "--subscription-group-id",
                "sub_1",
                "--pack-id",
                "pack_1",
                "--sort-order",
                "20",
            ]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(subscription_pack_output, "sub_1\tpack_1\t20");

        let subscription_pack_remove = execute_with_client(
            Cli::parse_from([
                "msm",
                "metadata",
                "subscription-groups",
                "packs",
                "remove",
                "--subscription-group-id",
                "sub_1",
                "--pack-id",
                "pack_1",
            ]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(subscription_pack_remove, "removed sub_1 pack_1");
    }

    #[tokio::test]
    async fn executes_portability_export_import_commands() {
        let client = FakeClient::default();
        let export_output = execute_with_client(
            Cli::parse_from([
                "msm",
                "portability",
                "export",
                "--user-id",
                "user_1",
                "--output",
                "-",
            ]),
            &client,
        )
        .await
        .unwrap();
        let export: serde_json::Value = serde_json::from_str(&export_output).unwrap();
        assert_eq!(export["user"]["id"], "user_1");

        let file = std::env::temp_dir().join("msm-portable-user-export.json");
        std::fs::write(&file, export_output).unwrap();
        let import_output = execute_with_client(
            Cli::parse_from([
                "msm",
                "portability",
                "import",
                "--tenant-id",
                "tenant_2",
                "--file",
                file.to_str().unwrap(),
            ]),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(import_output, "imported portable user data into tenant_2");
        assert_eq!(
            client
                .imported_user_data
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .tenant_id,
            "tenant_2"
        );
    }

    type TenantSettingsUpdateCall = (String, String, Option<String>, bool);

    #[derive(Default)]
    struct FakeClient {
        imported: Mutex<Option<ImportPackPayload>>,
        provider_import_plan: Mutex<Option<CreateProviderImportPlanPayload>>,
        created_provider_import_job: Mutex<Option<CreateProviderImportJobPayload>>,
        renamed_pack: Mutex<Option<crate::client::UpdatePackPayload>>,
        deleted_pack: Mutex<Option<String>>,
        created_pat: Mutex<Option<CreatePersonalAccessTokenPayload>>,
        revoked_pat: Mutex<Option<String>>,
        created_folder: Mutex<Option<CreateFolderPayload>>,
        created_tag: Mutex<Option<CreateTagPayload>>,
        created_subscription_group: Mutex<Option<CreateSubscriptionGroupPayload>>,
        added_folder_pack: Mutex<Option<(String, String, i64)>>,
        removed_folder_pack: Mutex<Option<(String, String)>>,
        added_pack_tag: Mutex<Option<(String, String)>>,
        removed_pack_tag: Mutex<Option<(String, String)>>,
        added_subscription_group_pack: Mutex<Option<(String, String, i64)>>,
        removed_subscription_group_pack: Mutex<Option<(String, String)>>,
        created_subscription_access_token: Mutex<Option<CreateSubscriptionAccessTokenPayload>>,
        rotated_subscription_access_token: Mutex<Option<String>>,
        revoked_subscription_access_token: Mutex<Option<String>>,
        upserted_tenant_member: Mutex<Option<(String, String, String)>>,
        updated_tenant_settings: Mutex<Option<TenantSettingsUpdateCall>>,
        updated_tenant_user_status: Mutex<Option<(String, String, bool)>>,
        upserted_tenant_role: Mutex<Option<TenantRoleUpsertCall>>,
        upserted_oidc_provider: Mutex<Option<OidcProviderUpsertCall>>,
        deleted_oidc_provider: Mutex<Option<(String, String)>>,
        listed_provider_config_tenant: Mutex<Option<String>>,
        upserted_provider_config: Mutex<Option<ProviderConfigUpsertCall>>,
        deleted_provider_config: Mutex<Option<String>>,
        created_export_target: Mutex<Option<CreateExportTargetPayload>>,
        created_export_job: Mutex<Option<CreateExportJobPayload>>,
        imported_user_data: Mutex<Option<ImportUserDataPayload>>,
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

        async fn export_user_data(&self, _user_id: &str) -> CliResult<serde_json::Value> {
            Ok(serde_json::json!({
                "version": 1,
                "exportedAt": "2026-05-10T00:00:00Z",
                "user": {
                    "id": "user_1",
                    "email": "leko@example.com",
                    "displayName": "Leko"
                },
                "packs": [],
                "subscriptionGroups": []
            }))
        }

        async fn import_user_data(&self, payload: ImportUserDataPayload) -> CliResult<()> {
            *self.imported_user_data.lock().unwrap() = Some(payload);
            Ok(())
        }

        async fn create_provider_import_plan(
            &self,
            payload: CreateProviderImportPlanPayload,
        ) -> CliResult<ProviderImportPlan> {
            *self.provider_import_plan.lock().unwrap() = Some(payload);
            Ok(ProviderImportPlan {
                provider_id: "line-stickers".to_owned(),
                remote_id: "line_cats".to_owned(),
                metadata_request: ProviderHttpRequestPlan {
                    method: "GET".to_owned(),
                    url: "https://store.line.me/stickershop/product/line_cats/en".to_owned(),
                    redacted_headers: vec![],
                },
                asset_strategy: "directRemoteUrls".to_owned(),
            })
        }

        async fn create_provider_import_job(
            &self,
            payload: CreateProviderImportJobPayload,
        ) -> CliResult<ProviderImportJob> {
            *self.created_provider_import_job.lock().unwrap() = Some(payload);
            Ok(sample_provider_import_job())
        }

        async fn get_provider_import_job(&self, _job_id: &str) -> CliResult<ProviderImportJob> {
            Ok(sample_provider_import_job())
        }

        async fn list_provider_import_job_events(
            &self,
            _job_id: &str,
        ) -> CliResult<Vec<ProviderImportJobEvent>> {
            Ok(vec![ProviderImportJobEvent {
                job_id: "provider_job_1".to_owned(),
                sequence: 1,
                level: "info".to_owned(),
                stage: "queued".to_owned(),
                message: "provider import job queued".to_owned(),
                metadata: serde_json::json!({}),
                created_at: "2026-05-10T00:00:00Z".to_owned(),
            }])
        }

        async fn list_provider_configs(&self, tenant_id: &str) -> CliResult<Vec<ProviderConfig>> {
            *self.listed_provider_config_tenant.lock().unwrap() = Some(tenant_id.to_owned());
            Ok(vec![sample_provider_config(
                "provider_telegram",
                tenant_id,
                "telegram",
                "Telegram Import Bot",
                true,
            )])
        }

        async fn upsert_provider_config(
            &self,
            config_id: &str,
            payload: UpsertProviderConfigPayload,
        ) -> CliResult<ProviderConfig> {
            *self.upserted_provider_config.lock().unwrap() =
                Some((config_id.to_owned(), payload.clone()));
            Ok(sample_provider_config(
                config_id,
                &payload.tenant_id,
                &payload.provider_id,
                &payload.name,
                payload.is_enabled,
            ))
        }

        async fn delete_provider_config(&self, config_id: &str) -> CliResult<()> {
            *self.deleted_provider_config.lock().unwrap() = Some(config_id.to_owned());
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

        async fn get_pat_scope_policy(&self, user_id: &str) -> CliResult<PatScopePolicy> {
            Ok(PatScopePolicy {
                user_id: user_id.to_owned(),
                allowed_scopes: vec!["pack.read".to_owned(), "pat.manage".to_owned()],
            })
        }

        async fn revoke_pat(&self, token_id: &str) -> CliResult<()> {
            *self.revoked_pat.lock().unwrap() = Some(token_id.to_owned());
            Ok(())
        }

        async fn list_tenant_members(&self, _tenant_id: &str) -> CliResult<Vec<TenantMember>> {
            Ok(vec![
                sample_tenant_member("user_1", "admin"),
                sample_tenant_member("user_2", "user"),
            ])
        }

        async fn set_tenant_member_role(
            &self,
            tenant_id: &str,
            user_id: &str,
            role: &str,
        ) -> CliResult<TenantMember> {
            *self.upserted_tenant_member.lock().unwrap() =
                Some((tenant_id.to_owned(), user_id.to_owned(), role.to_owned()));
            Ok(sample_tenant_member(user_id, role))
        }

        async fn get_tenant_settings(&self, _tenant_id: &str) -> CliResult<TenantSettings> {
            Ok(sample_tenant_settings(
                "tenant_1",
                "Tenant",
                Some("https://cdn.example.test/msm"),
                true,
            ))
        }

        async fn update_tenant_settings(
            &self,
            tenant_id: &str,
            payload: UpdateTenantSettingsPayload,
        ) -> CliResult<TenantSettings> {
            *self.updated_tenant_settings.lock().unwrap() = Some((
                tenant_id.to_owned(),
                payload.name.clone(),
                payload.public_asset_url.clone(),
                payload.local_registration_enabled,
            ));
            Ok(sample_tenant_settings(
                tenant_id,
                &payload.name,
                payload.public_asset_url.as_deref(),
                payload.local_registration_enabled,
            ))
        }

        async fn update_tenant_user_status(
            &self,
            tenant_id: &str,
            user_id: &str,
            payload: UpdateTenantUserStatusPayload,
        ) -> CliResult<TenantUser> {
            *self.updated_tenant_user_status.lock().unwrap() = Some((
                tenant_id.to_owned(),
                user_id.to_owned(),
                payload.is_disabled,
            ));
            Ok(sample_tenant_user(user_id, payload.is_disabled))
        }

        async fn list_tenant_roles(&self, _tenant_id: &str) -> CliResult<Vec<TenantRole>> {
            Ok(vec![sample_tenant_role(
                "role_editor",
                "tenant_1",
                "Editors",
                vec!["pack.read".to_owned(), "pack.update".to_owned()],
            )])
        }

        async fn upsert_tenant_role(
            &self,
            tenant_id: &str,
            role_id: &str,
            payload: UpsertTenantRolePayload,
        ) -> CliResult<TenantRole> {
            *self.upserted_tenant_role.lock().unwrap() = Some((
                tenant_id.to_owned(),
                role_id.to_owned(),
                payload.name.clone(),
                payload.permissions.clone(),
            ));
            Ok(sample_tenant_role(
                role_id,
                tenant_id,
                &payload.name,
                payload.permissions,
            ))
        }

        async fn list_oidc_providers(&self, _tenant_id: &str) -> CliResult<Vec<OidcProvider>> {
            Ok(vec![sample_oidc_provider(
                "google",
                "tenant_1",
                "Google Workspace",
                "https://accounts.google.com",
                true,
                true,
            )])
        }

        async fn upsert_oidc_provider(
            &self,
            tenant_id: &str,
            provider_id: &str,
            payload: UpsertOidcProviderPayload,
        ) -> CliResult<OidcProvider> {
            *self.upserted_oidc_provider.lock().unwrap() = Some((
                tenant_id.to_owned(),
                provider_id.to_owned(),
                payload.clone(),
            ));
            Ok(sample_oidc_provider(
                provider_id,
                tenant_id,
                &payload.display_name,
                &payload.issuer_url,
                payload.is_enabled,
                payload.allow_registration,
            ))
        }

        async fn delete_oidc_provider(&self, tenant_id: &str, provider_id: &str) -> CliResult<()> {
            *self.deleted_oidc_provider.lock().unwrap() =
                Some((tenant_id.to_owned(), provider_id.to_owned()));
            Ok(())
        }

        async fn create_folder(&self, payload: CreateFolderPayload) -> CliResult<Folder> {
            *self.created_folder.lock().unwrap() = Some(payload);
            Ok(sample_folder())
        }

        async fn list_folders(
            &self,
            _tenant_id: &str,
            _owner_user_id: &str,
        ) -> CliResult<Vec<Folder>> {
            Ok(vec![sample_folder()])
        }

        async fn create_tag(&self, payload: CreateTagPayload) -> CliResult<Tag> {
            *self.created_tag.lock().unwrap() = Some(payload);
            Ok(sample_tag())
        }

        async fn list_tags(&self, _tenant_id: &str) -> CliResult<Vec<Tag>> {
            Ok(vec![sample_tag()])
        }

        async fn create_subscription_group(
            &self,
            payload: CreateSubscriptionGroupPayload,
        ) -> CliResult<SubscriptionGroup> {
            *self.created_subscription_group.lock().unwrap() = Some(payload);
            Ok(sample_subscription_group())
        }

        async fn list_subscription_groups(
            &self,
            _tenant_id: &str,
            _owner_user_id: &str,
        ) -> CliResult<Vec<SubscriptionGroup>> {
            Ok(vec![sample_subscription_group()])
        }

        async fn add_pack_to_folder(
            &self,
            folder_id: &str,
            pack_id: &str,
            sort_order: i64,
        ) -> CliResult<FolderPack> {
            *self.added_folder_pack.lock().unwrap() =
                Some((folder_id.to_owned(), pack_id.to_owned(), sort_order));
            Ok(sample_folder_pack())
        }

        async fn list_folder_pack_ids(&self, _folder_id: &str) -> CliResult<Vec<String>> {
            Ok(vec!["pack_1".to_owned()])
        }

        async fn remove_pack_from_folder(&self, folder_id: &str, pack_id: &str) -> CliResult<()> {
            *self.removed_folder_pack.lock().unwrap() =
                Some((folder_id.to_owned(), pack_id.to_owned()));
            Ok(())
        }

        async fn add_tag_to_pack(&self, pack_id: &str, tag_id: &str) -> CliResult<PackTag> {
            *self.added_pack_tag.lock().unwrap() = Some((pack_id.to_owned(), tag_id.to_owned()));
            Ok(sample_pack_tag())
        }

        async fn list_pack_tag_ids(&self, _pack_id: &str) -> CliResult<Vec<String>> {
            Ok(vec!["tag_1".to_owned()])
        }

        async fn remove_tag_from_pack(&self, pack_id: &str, tag_id: &str) -> CliResult<()> {
            *self.removed_pack_tag.lock().unwrap() = Some((pack_id.to_owned(), tag_id.to_owned()));
            Ok(())
        }

        async fn add_pack_to_subscription_group(
            &self,
            subscription_group_id: &str,
            pack_id: &str,
            sort_order: i64,
        ) -> CliResult<SubscriptionGroupPack> {
            *self.added_subscription_group_pack.lock().unwrap() = Some((
                subscription_group_id.to_owned(),
                pack_id.to_owned(),
                sort_order,
            ));
            Ok(sample_subscription_group_pack())
        }

        async fn list_subscription_group_pack_ids(
            &self,
            _subscription_group_id: &str,
        ) -> CliResult<Vec<String>> {
            Ok(vec!["pack_1".to_owned()])
        }

        async fn remove_pack_from_subscription_group(
            &self,
            subscription_group_id: &str,
            pack_id: &str,
        ) -> CliResult<()> {
            *self.removed_subscription_group_pack.lock().unwrap() =
                Some((subscription_group_id.to_owned(), pack_id.to_owned()));
            Ok(())
        }

        async fn create_subscription_access_token(
            &self,
            payload: CreateSubscriptionAccessTokenPayload,
        ) -> CliResult<CreatedSubscriptionAccessToken> {
            *self.created_subscription_access_token.lock().unwrap() = Some(payload);
            Ok(sample_created_subscription_access_token())
        }

        async fn list_subscription_access_tokens(
            &self,
            _user_id: &str,
        ) -> CliResult<Vec<SubscriptionAccessToken>> {
            Ok(vec![sample_subscription_access_token()])
        }

        async fn rotate_subscription_access_token(
            &self,
            token_id: &str,
        ) -> CliResult<CreatedSubscriptionAccessToken> {
            *self.rotated_subscription_access_token.lock().unwrap() = Some(token_id.to_owned());
            Ok(sample_created_subscription_access_token())
        }

        async fn revoke_subscription_access_token(&self, token_id: &str) -> CliResult<()> {
            *self.revoked_subscription_access_token.lock().unwrap() = Some(token_id.to_owned());
            Ok(())
        }

        async fn list_export_target_kinds(&self) -> CliResult<Vec<ExportTargetKind>> {
            Ok(vec![ExportTargetKind {
                kind: "telegram".to_owned(),
                display_name: "Telegram".to_owned(),
                supports_remote_publication: true,
                supports_media_conversion: true,
                requires_credentials: true,
            }])
        }

        async fn list_export_targets(&self, _tenant_id: &str) -> CliResult<Vec<ExportTarget>> {
            Ok(vec![sample_export_target()])
        }

        async fn create_export_target(
            &self,
            payload: CreateExportTargetPayload,
        ) -> CliResult<ExportTarget> {
            *self.created_export_target.lock().unwrap() = Some(payload);
            Ok(sample_export_target())
        }

        async fn update_export_target(
            &self,
            _payload: UpdateExportTargetPayload,
        ) -> CliResult<ExportTarget> {
            Ok(sample_export_target())
        }

        async fn delete_export_target(&self, _target_id: &str) -> CliResult<()> {
            Ok(())
        }

        async fn create_export_job(&self, payload: CreateExportJobPayload) -> CliResult<ExportJob> {
            *self.created_export_job.lock().unwrap() = Some(payload);
            Ok(sample_export_job())
        }

        async fn get_export_job(&self, _job_id: &str) -> CliResult<ExportJob> {
            Ok(sample_export_job())
        }

        async fn requeue_export_job(&self, _job_id: &str) -> CliResult<ExportJob> {
            Ok(sample_export_job())
        }

        async fn list_export_job_events(&self, _job_id: &str) -> CliResult<Vec<ExportJobEvent>> {
            Ok(vec![ExportJobEvent {
                job_id: "job_1".to_owned(),
                sequence: 1,
                level: "info".to_owned(),
                stage: "queued".to_owned(),
                message: "job queued".to_owned(),
                metadata: serde_json::json!({}),
                created_at: "2026-05-07T00:00:00Z".to_owned(),
            }])
        }

        async fn list_telegram_publications(
            &self,
            _pack_id: &str,
        ) -> CliResult<Vec<TelegramPublication>> {
            Ok(vec![sample_telegram_publication()])
        }

        async fn get_telegram_publication(
            &self,
            _publication_id: &str,
        ) -> CliResult<TelegramPublication> {
            Ok(sample_telegram_publication())
        }
    }

    fn sample_provider_import_job() -> ProviderImportJob {
        ProviderImportJob {
            id: "provider_job_1".to_owned(),
            tenant_id: "tenant_1".to_owned(),
            owner_user_id: "user_1".to_owned(),
            provider_id: "line-stickers".to_owned(),
            remote_id: "line_cats".to_owned(),
            target_pack_id: Some("pack_line_cats".to_owned()),
            status: "queued".to_owned(),
            request: serde_json::json!({ "providerId": "line-stickers" }),
            result: None,
            error_summary: None,
            attempt_count: 0,
            max_attempts: 3,
            next_attempt_at: None,
            created_at: "2026-05-10T00:00:00Z".to_owned(),
            updated_at: "2026-05-10T00:00:00Z".to_owned(),
        }
    }

    fn sample_export_target() -> ExportTarget {
        ExportTarget {
            id: "target_telegram".to_owned(),
            tenant_id: "tenant_1".to_owned(),
            kind: "telegram".to_owned(),
            name: "Telegram".to_owned(),
            config: serde_json::json!({ "botToken": "<redacted>" }),
            is_enabled: true,
            created_at: "2026-05-07T00:00:00Z".to_owned(),
            updated_at: "2026-05-07T00:00:00Z".to_owned(),
        }
    }

    fn sample_provider_config(
        id: &str,
        tenant_id: &str,
        provider_id: &str,
        name: &str,
        is_enabled: bool,
    ) -> ProviderConfig {
        ProviderConfig {
            id: id.to_owned(),
            tenant_id: tenant_id.to_owned(),
            provider_id: provider_id.to_owned(),
            name: name.to_owned(),
            config: serde_json::json!({ "botToken": "<redacted>" }),
            is_enabled,
            created_at: "2026-05-10T00:00:00Z".to_owned(),
            updated_at: "2026-05-10T00:00:00Z".to_owned(),
        }
    }

    fn sample_oidc_provider(
        id: &str,
        tenant_id: &str,
        display_name: &str,
        issuer_url: &str,
        is_enabled: bool,
        allow_registration: bool,
    ) -> OidcProvider {
        OidcProvider {
            id: id.to_owned(),
            tenant_id: tenant_id.to_owned(),
            display_name: display_name.to_owned(),
            issuer_url: issuer_url.to_owned(),
            client_id: "client_1".to_owned(),
            client_secret: "[redacted]".to_owned(),
            scopes: vec!["openid".to_owned(), "email".to_owned()],
            is_enabled,
            allow_registration,
            created_at: "2026-05-10T00:00:00Z".to_owned(),
            updated_at: "2026-05-10T00:00:00Z".to_owned(),
        }
    }

    fn sample_folder() -> Folder {
        Folder {
            id: "folder_1".to_owned(),
            tenant_id: "tenant_1".to_owned(),
            owner_user_id: "user_1".to_owned(),
            name: "Favorites".to_owned(),
            created_at: "2026-05-09T00:00:00Z".to_owned(),
        }
    }

    fn sample_tenant_member(user_id: &str, role: &str) -> TenantMember {
        TenantMember {
            tenant_id: "tenant_1".to_owned(),
            user_id: user_id.to_owned(),
            role: role.to_owned(),
            created_at: "2026-05-09T00:00:00Z".to_owned(),
        }
    }

    fn sample_tenant_settings(
        tenant_id: &str,
        name: &str,
        public_asset_url: Option<&str>,
        local_registration_enabled: bool,
    ) -> TenantSettings {
        TenantSettings {
            tenant_id: tenant_id.to_owned(),
            name: name.to_owned(),
            public_asset_url: public_asset_url.map(str::to_owned),
            local_registration_enabled,
            created_at: "2026-05-09T00:00:00Z".to_owned(),
        }
    }

    fn sample_tenant_user(user_id: &str, is_disabled: bool) -> TenantUser {
        TenantUser {
            id: user_id.to_owned(),
            email: "user@example.com".to_owned(),
            display_name: "User".to_owned(),
            is_disabled,
            created_at: "2026-05-09T00:00:00Z".to_owned(),
        }
    }

    fn sample_tenant_role(
        role_id: &str,
        tenant_id: &str,
        name: &str,
        permissions: Vec<String>,
    ) -> TenantRole {
        TenantRole {
            id: role_id.to_owned(),
            tenant_id: Some(tenant_id.to_owned()),
            name: name.to_owned(),
            permissions,
            created_at: "2026-05-09T00:00:00Z".to_owned(),
        }
    }

    fn sample_tag() -> Tag {
        Tag {
            id: "tag_1".to_owned(),
            tenant_id: "tenant_1".to_owned(),
            name: "cute".to_owned(),
            created_at: "2026-05-09T00:00:00Z".to_owned(),
        }
    }

    fn sample_subscription_group() -> SubscriptionGroup {
        SubscriptionGroup {
            id: "sub_1".to_owned(),
            tenant_id: "tenant_1".to_owned(),
            owner_user_id: "user_1".to_owned(),
            title: "Weekly".to_owned(),
            visibility: PackVisibility::Private,
            created_at: "2026-05-09T00:00:00Z".to_owned(),
        }
    }

    fn sample_folder_pack() -> FolderPack {
        FolderPack {
            folder_id: "folder_1".to_owned(),
            pack_id: "pack_1".to_owned(),
            sort_order: 10,
        }
    }

    fn sample_pack_tag() -> PackTag {
        PackTag {
            pack_id: "pack_1".to_owned(),
            tag_id: "tag_1".to_owned(),
        }
    }

    fn sample_subscription_group_pack() -> SubscriptionGroupPack {
        SubscriptionGroupPack {
            subscription_group_id: "sub_1".to_owned(),
            pack_id: "pack_1".to_owned(),
            sort_order: 20,
        }
    }

    fn sample_subscription_access_token() -> SubscriptionAccessToken {
        SubscriptionAccessToken {
            id: "packlink".to_owned(),
            tenant_id: "tenant_1".to_owned(),
            owner_user_id: "user_1".to_owned(),
            resource_type: SubscriptionAccessResourceType::Pack,
            resource_id: "pack_1".to_owned(),
            revoked_at: None,
            created_at: "2026-05-09T00:00:00Z".to_owned(),
            updated_at: "2026-05-09T00:00:00Z".to_owned(),
        }
    }

    fn sample_created_subscription_access_token() -> CreatedSubscriptionAccessToken {
        CreatedSubscriptionAccessToken {
            id: "packlink".to_owned(),
            tenant_id: "tenant_1".to_owned(),
            owner_user_id: "user_1".to_owned(),
            resource_type: SubscriptionAccessResourceType::Pack,
            resource_id: "pack_1".to_owned(),
            token: "msm_sub_packlink_secret".to_owned(),
            revoked_at: None,
            created_at: "2026-05-09T00:00:00Z".to_owned(),
            updated_at: "2026-05-09T00:00:00Z".to_owned(),
        }
    }

    fn sample_export_job() -> ExportJob {
        ExportJob {
            id: "job_1".to_owned(),
            tenant_id: "tenant_1".to_owned(),
            owner_user_id: "user_1".to_owned(),
            source_pack_id: "pack_1".to_owned(),
            target_id: "target_telegram".to_owned(),
            status: "queued".to_owned(),
            request: serde_json::json!({ "options": {} }),
            result: None,
            error_summary: None,
            attempt_count: 0,
            max_attempts: 3,
            next_attempt_at: None,
            created_at: "2026-05-07T00:00:00Z".to_owned(),
            updated_at: "2026-05-07T00:00:00Z".to_owned(),
        }
    }

    fn sample_telegram_publication() -> TelegramPublication {
        TelegramPublication {
            id: "telegram_pub_1".to_owned(),
            pack_id: "pack_1".to_owned(),
            target_id: "target_telegram".to_owned(),
            job_id: "job_1".to_owned(),
            sticker_set_name: "sample_by_msm_bot".to_owned(),
            sticker_set_url: "https://t.me/addstickers/sample_by_msm_bot".to_owned(),
            sticker_count: 1,
            sticker_type: "regular".to_owned(),
            created_at: "2026-05-07T00:00:00Z".to_owned(),
            updated_at: "2026-05-07T00:00:00Z".to_owned(),
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

use msm_domain::StickerPack;

use crate::{
    client::{
        CreatedPersonalAccessToken, CreatedSubscriptionAccessToken, ExportJob, ExportJobEvent,
        ExportTarget, ExportTargetKind, Folder, FolderPack, PackTag, PersonalAccessToken,
        SubscriptionAccessToken, SubscriptionGroup, SubscriptionGroupPack, Tag,
        TelegramPublication,
    },
    command::OutputFormat,
    CliResult,
};

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct HealthResponse {
    pub status: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResponse {
    pub status: &'static str,
    pub pack_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevokePatResponse {
    pub status: &'static str,
    pub token_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionLinkMutationResponse {
    pub status: &'static str,
    pub token_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackMutationResponse {
    pub status: &'static str,
    pub pack_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MembershipMutationResponse {
    pub status: &'static str,
    pub left_id: String,
    pub right_id: String,
}

/// Formats a health response.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_health(format: OutputFormat, response: &HealthResponse) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(response.status.clone()),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(response)?),
    }
}

/// Formats a pack list response.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_pack_list(format: OutputFormat, packs: &[StickerPack]) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(packs
            .iter()
            .map(|pack| format!("{}\t{}", pack.id, pack.title))
            .collect::<Vec<_>>()
            .join("\n")),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(packs)?),
    }
}

/// Formats an import response.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_import(format: OutputFormat, pack_id: &str) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(format!("imported {pack_id}")),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(&ImportResponse {
            status: "imported",
            pack_id: pack_id.to_owned(),
        })?),
    }
}

/// Formats a pack rename response.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_pack_rename(format: OutputFormat, pack_id: &str) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(format!("renamed {pack_id}")),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(&PackMutationResponse {
            status: "renamed",
            pack_id: pack_id.to_owned(),
        })?),
    }
}

/// Formats a pack delete response.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_pack_delete(format: OutputFormat, pack_id: &str) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(format!("deleted {pack_id}")),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(&PackMutationResponse {
            status: "deleted",
            pack_id: pack_id.to_owned(),
        })?),
    }
}

/// Formats an exported sticker pack.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_export(pack: &StickerPack) -> CliResult<String> {
    Ok(serde_json::to_string_pretty(pack)?)
}

/// Formats a PAT create response.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_pat_create(
    format: OutputFormat,
    response: &CreatedPersonalAccessToken,
) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(format!("created {}\n{}", response.id, response.token)),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(response)?),
    }
}

/// Formats PAT list responses.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_pat_list(format: OutputFormat, tokens: &[PersonalAccessToken]) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(tokens
            .iter()
            .map(|token| format!("{}\t{}\t{}", token.id, token.name, token.scopes.join(",")))
            .collect::<Vec<_>>()
            .join("\n")),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(tokens)?),
    }
}

/// Formats a PAT revoke response.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_pat_revoke(format: OutputFormat, token_id: &str) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(format!("revoked {token_id}")),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(&RevokePatResponse {
            status: "revoked",
            token_id: token_id.to_owned(),
        })?),
    }
}

/// Formats a subscription access token create/rotate response.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_subscription_link_secret(
    format: OutputFormat,
    response: &CreatedSubscriptionAccessToken,
) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(format!("{}\t{}", response.id, response.token)),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(response)?),
    }
}

/// Formats subscription access token metadata responses.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_subscription_links(
    format: OutputFormat,
    links: &[SubscriptionAccessToken],
) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(links
            .iter()
            .map(subscription_link_line)
            .collect::<Vec<_>>()
            .join("\n")),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(links)?),
    }
}

/// Formats a subscription access token revoke response.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_subscription_link_revoke(format: OutputFormat, token_id: &str) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(format!("revoked {token_id}")),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(
            &SubscriptionLinkMutationResponse {
                status: "revoked",
                token_id: token_id.to_owned(),
            },
        )?),
    }
}

/// Formats folder list responses.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_folders(format: OutputFormat, folders: &[Folder]) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(folders
            .iter()
            .map(folder_line)
            .collect::<Vec<_>>()
            .join("\n")),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(folders)?),
    }
}

/// Formats one folder response.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_folder(format: OutputFormat, folder: &Folder) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(folder_line(folder)),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(folder)?),
    }
}

/// Formats folder pack ID list responses.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_pack_ids(format: OutputFormat, pack_ids: &[String]) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(pack_ids.join("\n")),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(pack_ids)?),
    }
}

/// Formats tag ID list responses.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_tag_ids(format: OutputFormat, tag_ids: &[String]) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(tag_ids.join("\n")),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(tag_ids)?),
    }
}

/// Formats one folder-pack response.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_folder_pack(format: OutputFormat, link: &FolderPack) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(folder_pack_line(link)),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(link)?),
    }
}

/// Formats one pack-tag response.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_pack_tag(format: OutputFormat, link: &PackTag) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(pack_tag_line(link)),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(link)?),
    }
}

/// Formats one subscription-group-pack response.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_subscription_group_pack(
    format: OutputFormat,
    link: &SubscriptionGroupPack,
) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(subscription_group_pack_line(link)),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(link)?),
    }
}

/// Formats a removed membership response.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_membership_remove(
    format: OutputFormat,
    left_id: &str,
    right_id: &str,
) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(format!("removed {left_id} {right_id}")),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(&MembershipMutationResponse {
            status: "removed",
            left_id: left_id.to_owned(),
            right_id: right_id.to_owned(),
        })?),
    }
}

/// Formats tag list responses.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_tags(format: OutputFormat, tags: &[Tag]) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(tags.iter().map(tag_line).collect::<Vec<_>>().join("\n")),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(tags)?),
    }
}

/// Formats one tag response.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_tag(format: OutputFormat, tag: &Tag) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(tag_line(tag)),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(tag)?),
    }
}

/// Formats subscription group list responses.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_subscription_groups(
    format: OutputFormat,
    groups: &[SubscriptionGroup],
) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(groups.iter().map(group_line).collect::<Vec<_>>().join("\n")),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(groups)?),
    }
}

/// Formats one subscription group response.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_subscription_group(
    format: OutputFormat,
    group: &SubscriptionGroup,
) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(group_line(group)),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(group)?),
    }
}

/// Formats export target kind responses.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_export_target_kinds(
    format: OutputFormat,
    kinds: &[ExportTargetKind],
) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(kinds
            .iter()
            .map(|kind| format!("{}\t{}", kind.kind, kind.display_name))
            .collect::<Vec<_>>()
            .join("\n")),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(kinds)?),
    }
}

/// Formats export target list responses.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_export_targets(format: OutputFormat, targets: &[ExportTarget]) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(targets
            .iter()
            .map(|target| {
                let state = if target.is_enabled {
                    "enabled"
                } else {
                    "disabled"
                };
                format!("{}\t{}\t{}\t{state}", target.id, target.kind, target.name)
            })
            .collect::<Vec<_>>()
            .join("\n")),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(targets)?),
    }
}

/// Formats an export target mutation response.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_export_target(format: OutputFormat, target: &ExportTarget) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(format!("created {}", target.id)),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(target)?),
    }
}

/// Formats an export job response.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_export_job(format: OutputFormat, job: &ExportJob) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(format!(
            "{}\t{}\t{}/{}",
            job.id, job.status, job.attempt_count, job.max_attempts
        )),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(job)?),
    }
}

/// Formats export job events.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_export_job_events(
    format: OutputFormat,
    events: &[ExportJobEvent],
) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(events
            .iter()
            .map(|event| {
                format!(
                    "{}\t{}\t{}\t{}",
                    event.sequence, event.level, event.stage, event.message
                )
            })
            .collect::<Vec<_>>()
            .join("\n")),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(events)?),
    }
}

/// Formats Telegram publication list responses.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_telegram_publications(
    format: OutputFormat,
    publications: &[TelegramPublication],
) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(publications
            .iter()
            .map(telegram_publication_line)
            .collect::<Vec<_>>()
            .join("\n")),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(publications)?),
    }
}

/// Formats one Telegram publication response.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_telegram_publication(
    format: OutputFormat,
    publication: &TelegramPublication,
) -> CliResult<String> {
    match format {
        OutputFormat::Human => Ok(telegram_publication_line(publication)),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(publication)?),
    }
}

fn telegram_publication_line(publication: &TelegramPublication) -> String {
    format!(
        "{}\t{}\t{}",
        publication.id, publication.sticker_set_name, publication.sticker_set_url
    )
}

fn folder_line(folder: &Folder) -> String {
    format!("{}\t{}", folder.id, folder.name)
}

fn folder_pack_line(link: &FolderPack) -> String {
    format!("{}\t{}\t{}", link.folder_id, link.pack_id, link.sort_order)
}

fn tag_line(tag: &Tag) -> String {
    format!("{}\t{}", tag.id, tag.name)
}

fn pack_tag_line(link: &PackTag) -> String {
    format!("{}\t{}", link.pack_id, link.tag_id)
}

fn group_line(group: &SubscriptionGroup) -> String {
    format!(
        "{}\t{}\t{}",
        group.id,
        group.title,
        group.visibility.as_str()
    )
}

fn subscription_group_pack_line(link: &SubscriptionGroupPack) -> String {
    format!(
        "{}\t{}\t{}",
        link.subscription_group_id, link.pack_id, link.sort_order
    )
}

fn subscription_link_line(link: &SubscriptionAccessToken) -> String {
    format!(
        "{}\t{:?}\t{}\t{}",
        link.id,
        link.resource_type,
        link.resource_id,
        link.revoked_at.is_some()
    )
}

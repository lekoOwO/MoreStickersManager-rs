use msm_domain::StickerPack;

use crate::{
    client::{
        CreatedPersonalAccessToken, ExportJob, ExportJobEvent, ExportTarget, ExportTargetKind,
        PersonalAccessToken,
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
pub struct PackMutationResponse {
    pub status: &'static str,
    pub pack_id: String,
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
        OutputFormat::Human => Ok(format!("{}\t{}", job.id, job.status)),
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

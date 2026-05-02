use msm_domain::StickerPack;

use crate::{CliResult, command::OutputFormat};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct HealthResponse {
    pub status: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResponse {
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

/// Formats an exported sticker pack.
///
/// # Errors
///
/// Returns an error when JSON serialization fails.
pub fn format_export(pack: &StickerPack) -> CliResult<String> {
    Ok(serde_json::to_string_pretty(pack)?)
}

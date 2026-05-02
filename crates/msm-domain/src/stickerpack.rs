use std::{collections::BTreeMap, fs, path::Path};

use crate::{DomainError, DomainResult};

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sticker {
    pub id: String,
    pub image: String,
    pub title: String,
    pub sticker_pack_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_animated: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StickerPack {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<Author>,
    pub logo: Sticker,
    pub stickers: Vec<Sticker>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub refresh_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_headers: Option<BTreeMap<String, String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicStickerPackMeta {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<Author>,
    pub logo: Sticker,
    pub dynamic: DynamicInfo,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicPackSetMeta {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<Author>,
    pub packs: Vec<DynamicStickerPackMeta>,
    pub refresh_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_headers: Option<BTreeMap<String, String>>,
}

impl StickerPack {
    /// Parses a `MoreStickers`-compatible sticker pack from JSON.
    ///
    /// # Errors
    ///
    /// Returns an error when the input is not valid JSON for the sticker pack schema.
    pub fn from_json_str(input: &str) -> DomainResult<Self> {
        Ok(serde_json::from_str(input)?)
    }

    /// Serializes the sticker pack as pretty JSON.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization fails.
    pub fn to_pretty_json(&self) -> DomainResult<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Reads a `.stickerpack` file from disk.
    ///
    /// # Errors
    ///
    /// Returns an error when the path extension is not `.stickerpack`, the file cannot be read,
    /// or the file content is not valid sticker pack JSON.
    pub fn read_stickerpack_file(path: impl AsRef<Path>) -> DomainResult<Self> {
        let path = path.as_ref();
        if path.extension().and_then(|value| value.to_str()) != Some("stickerpack") {
            return Err(DomainError::InvalidStickerPackExtension {
                path: path.to_path_buf(),
            });
        }

        let content = fs::read_to_string(path).map_err(serde_json::Error::io)?;
        Self::from_json_str(&content)
    }
}

impl DynamicPackSetMeta {
    /// Parses a `MoreStickers` dynamic pack set from JSON.
    ///
    /// # Errors
    ///
    /// Returns an error when the input is not valid JSON for the dynamic pack set schema.
    pub fn from_json_str(input: &str) -> DomainResult<Self> {
        Ok(serde_json::from_str(input)?)
    }

    /// Serializes the dynamic pack set as pretty JSON.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization fails.
    pub fn to_pretty_json(&self) -> DomainResult<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}

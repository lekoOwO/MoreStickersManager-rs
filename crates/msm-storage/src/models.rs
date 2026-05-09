use chrono::{DateTime, Utc};
use msm_domain::{Permission, StickerPack};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct TenantRecord {
    pub id: String,
    pub name: String,
    pub public_asset_url: Option<String>,
    pub local_registration_enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct UserRecord {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub is_disabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct TenantMemberRecord {
    pub tenant_id: String,
    pub user_id: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct RoleRecord {
    pub id: String,
    pub tenant_id: Option<String>,
    pub name: String,
    pub permissions: BTreeSet<Permission>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct OidcProviderConfigRecord {
    pub id: String,
    pub tenant_id: String,
    pub display_name: String,
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub scopes: BTreeSet<String>,
    pub is_enabled: bool,
    pub allow_registration: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewOidcProviderConfig<'a> {
    pub id: &'a str,
    pub tenant_id: &'a str,
    pub display_name: &'a str,
    pub issuer_url: &'a str,
    pub client_id: &'a str,
    pub client_secret: &'a str,
    pub scopes: &'a BTreeSet<String>,
    pub is_enabled: bool,
    pub allow_registration: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct OidcLoginStateRecord {
    pub id: String,
    pub tenant_id: String,
    pub provider_id: String,
    pub state_hash: String,
    pub nonce_hash: String,
    pub redirect_uri: String,
    pub expires_at: String,
    pub consumed_at: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreatedOidcLoginState {
    pub record: OidcLoginStateRecord,
    pub state: String,
    pub nonce: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct OidcUserLinkRecord {
    pub tenant_id: String,
    pub provider_id: String,
    pub provider_subject: String,
    pub user_id: String,
    pub email: String,
    pub display_name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct LocalUserCredentialRecord {
    pub user_id: String,
    pub password_hash: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct StickerPackRecord {
    pub id: String,
    pub tenant_id: String,
    pub owner_user_id: String,
    pub compatibility_id: String,
    pub title: String,
    pub visibility: PackVisibility,
    pub source_provider: Option<String>,
    pub sticker_pack: StickerPack,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct StickerRecord {
    pub id: String,
    pub pack_id: String,
    pub compatibility_id: String,
    pub title: String,
    pub asset_key: Option<String>,
    pub image_url: String,
    pub is_animated: Option<bool>,
    pub sort_order: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct FolderRecord {
    pub id: String,
    pub tenant_id: String,
    pub owner_user_id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct FolderPackRecord {
    pub folder_id: String,
    pub pack_id: String,
    pub sort_order: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct TagRecord {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct PackTagRecord {
    pub pack_id: String,
    pub tag_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewTag<'a> {
    pub id: &'a str,
    pub tenant_id: &'a str,
    pub name: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct SubscriptionGroupRecord {
    pub id: String,
    pub tenant_id: String,
    pub owner_user_id: String,
    pub title: String,
    pub visibility: PackVisibility,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct SubscriptionGroupPackRecord {
    pub subscription_group_id: String,
    pub pack_id: String,
    pub sort_order: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct PersonalAccessTokenRecord {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub token_hash: String,
    pub scopes: BTreeSet<Permission>,
    pub expires_at: Option<String>,
    pub revoked_at: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreatedPersonalAccessToken {
    pub record: PersonalAccessTokenRecord,
    pub token: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct WebSessionRecord {
    pub id: String,
    pub user_id: String,
    pub session_hash: String,
    pub expires_at: Option<String>,
    pub revoked_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreatedWebSession {
    pub record: WebSessionRecord,
    pub token: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct SubscriptionAccessTokenRecord {
    pub id: String,
    pub tenant_id: String,
    pub owner_user_id: String,
    pub resource_type: SubscriptionAccessResourceType,
    pub resource_id: String,
    pub token_hash: String,
    pub revoked_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreatedSubscriptionAccessToken {
    pub record: SubscriptionAccessTokenRecord,
    pub token: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum SubscriptionAccessResourceType {
    Pack,
    SubscriptionGroup,
}

impl SubscriptionAccessResourceType {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pack => "pack",
            Self::SubscriptionGroup => "subscription_group",
        }
    }

    #[must_use]
    pub fn from_storage(value: &str) -> Option<Self> {
        match value {
            "pack" => Some(Self::Pack),
            "subscription_group" => Some(Self::SubscriptionGroup),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct ExportTargetRecord {
    pub id: String,
    pub tenant_id: String,
    pub kind: String,
    pub name: String,
    pub config_json: String,
    pub is_enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewExportTarget<'a> {
    pub id: &'a str,
    pub tenant_id: &'a str,
    pub kind: &'a str,
    pub name: &'a str,
    pub config_json: &'a str,
    pub is_enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct ExportJobRecord {
    pub id: String,
    pub tenant_id: String,
    pub owner_user_id: String,
    pub source_pack_id: String,
    pub target_id: String,
    pub status: ExportJobStatus,
    pub request_json: String,
    pub result_json: Option<String>,
    pub error_summary: Option<String>,
    pub attempt_count: i64,
    pub max_attempts: i64,
    pub next_attempt_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewExportJob<'a> {
    pub id: &'a str,
    pub tenant_id: &'a str,
    pub owner_user_id: &'a str,
    pub source_pack_id: &'a str,
    pub target_id: &'a str,
    pub request_json: &'a str,
    pub max_attempts: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct ExportJobEventRecord {
    pub job_id: String,
    pub sequence: i64,
    pub level: String,
    pub stage: String,
    pub message: String,
    pub metadata_json: String,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewExportJobEvent<'a> {
    pub job_id: &'a str,
    pub sequence: i64,
    pub level: &'a str,
    pub stage: &'a str,
    pub message: &'a str,
    pub metadata_json: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct PreparedMediaAssetRecord {
    pub source_asset_hash: String,
    pub profile_key: String,
    pub output_asset_key: String,
    pub mime_type: String,
    pub width_px: i64,
    pub height_px: i64,
    pub duration_ms: Option<i64>,
    pub file_size_bytes: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewPreparedMediaAsset<'a> {
    pub source_asset_hash: &'a str,
    pub profile_key: &'a str,
    pub output_asset_key: &'a str,
    pub mime_type: &'a str,
    pub width_px: i64,
    pub height_px: i64,
    pub duration_ms: Option<i64>,
    pub file_size_bytes: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct TelegramPublicationRecord {
    pub id: String,
    pub pack_id: String,
    pub target_id: String,
    pub job_id: String,
    pub sticker_set_name: String,
    pub sticker_set_url: String,
    pub sticker_count: i64,
    pub sticker_type: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct TelegramStickerMappingRecord {
    pub id: String,
    pub publication_id: String,
    pub target_id: String,
    pub sticker_set_name: String,
    pub source_sticker_id: String,
    pub telegram_file_id: String,
    pub telegram_file_unique_id: String,
    pub position: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewTelegramPublication<'a> {
    pub id: &'a str,
    pub pack_id: &'a str,
    pub target_id: &'a str,
    pub job_id: &'a str,
    pub sticker_set_name: &'a str,
    pub sticker_set_url: &'a str,
    pub sticker_count: i64,
    pub sticker_type: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewTelegramStickerMapping<'a> {
    pub publication_id: &'a str,
    pub target_id: &'a str,
    pub sticker_set_name: &'a str,
    pub source_sticker_id: &'a str,
    pub telegram_file_id: &'a str,
    pub telegram_file_unique_id: &'a str,
    pub position: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum ExportJobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl ExportJobStatus {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    #[must_use]
    pub fn from_storage(value: &str) -> Option<Self> {
        match value {
            "queued" => Some(Self::Queued),
            "running" => Some(Self::Running),
            "succeeded" => Some(Self::Succeeded),
            "failed" => Some(Self::Failed),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum PackVisibility {
    Public,
    Private,
}

impl PackVisibility {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
        }
    }

    #[must_use]
    pub fn from_storage(value: &str) -> Option<Self> {
        match value {
            "public" => Some(Self::Public),
            "private" => Some(Self::Private),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PackVisibility;

    #[test]
    fn visibility_serializes_to_storage_value() {
        assert_eq!(PackVisibility::Public.as_str(), "public");
        assert_eq!(PackVisibility::Private.as_str(), "private");
    }
}

use chrono::{DateTime, Utc};
use msm_domain::StickerPack;

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct TenantRecord {
    pub id: String,
    pub name: String,
    pub public_asset_url: Option<String>,
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
pub struct SubscriptionGroupRecord {
    pub id: String,
    pub tenant_id: String,
    pub owner_user_id: String,
    pub title: String,
    pub visibility: PackVisibility,
    pub created_at: DateTime<Utc>,
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

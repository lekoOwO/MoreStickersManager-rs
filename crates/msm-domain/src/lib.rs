#![doc = "Domain models and pure helpers for MoreStickersManager-rs."]

pub mod error;
pub mod authz;
pub mod ids;
pub mod stickerpack;
pub mod url;

pub use authz::{
    evaluate_pack_access, evaluate_subscription_access, AccessContext, MemberAccess, PackAction,
    PackResource, Permission, PolicyDecision, PolicyReason, Principal, Role, SubscriptionAction,
    SubscriptionResource, Visibility,
};
pub use error::{DomainError, DomainResult};
pub use ids::{
    line_emoji_id, line_emoji_pack_id, line_sticker_id, line_sticker_pack_id, telegram_pack_id,
    telegram_sticker_id,
};
pub use stickerpack::{
    Author, DynamicInfo, DynamicPackSetMeta, DynamicStickerPackMeta, Sticker, StickerPack,
};
pub use url::{resolve_asset_url, AssetUrlConfig, AssetUrlInput};

use std::collections::BTreeMap;

use crate::{Author, DynamicInfo, DynamicPackSetMeta, DynamicStickerPackMeta, StickerPack};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubscriptionPackInput {
    pub pack: StickerPack,
    pub refresh_url: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubscriptionPayloadInput {
    pub id: String,
    pub version: Option<String>,
    pub title: Option<String>,
    pub author: Option<Author>,
    pub refresh_url: String,
    pub auth_headers: Option<BTreeMap<String, String>>,
    pub packs: Vec<SubscriptionPackInput>,
}

#[must_use]
pub fn build_dynamic_subscription_payload(input: SubscriptionPayloadInput) -> DynamicPackSetMeta {
    let packs = input
        .packs
        .into_iter()
        .map(|pack_input| DynamicStickerPackMeta {
            id: pack_input.pack.id,
            title: pack_input.pack.title,
            author: pack_input.pack.author,
            logo: pack_input.pack.logo,
            dynamic: DynamicInfo {
                version: input.version.clone(),
                refresh_url: pack_input.refresh_url,
                auth_headers: input.auth_headers.clone(),
            },
        })
        .collect();

    DynamicPackSetMeta {
        id: input.id,
        version: input.version,
        title: input.title,
        author: input.author,
        packs,
        refresh_url: input.refresh_url,
        auth_headers: input.auth_headers,
    }
}

#[must_use]
pub fn subscription_bearer_headers(secret: &str) -> BTreeMap<String, String> {
    BTreeMap::from([(
        "Authorization".to_owned(),
        format!("Bearer {}", secret.trim()),
    )])
}

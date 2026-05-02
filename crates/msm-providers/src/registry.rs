#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderMetadata {
    pub id: &'static str,
    pub display_name: &'static str,
    pub capabilities: &'static [ProviderCapability],
    pub status: ProviderStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderCapability {
    NormalizeFixture,
    FetchRemote,
    DownloadAssets,
    AnimatedStickers,
    EmojiPacks,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderStatus {
    Implemented,
    Planned,
}

pub const TELEGRAM_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::NormalizeFixture,
    ProviderCapability::AnimatedStickers,
];

pub const LINE_STICKER_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::NormalizeFixture,
    ProviderCapability::AnimatedStickers,
];

pub const LINE_EMOJI_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::NormalizeFixture,
    ProviderCapability::AnimatedStickers,
    ProviderCapability::EmojiPacks,
];

pub const PLANNED_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::FetchRemote,
    ProviderCapability::DownloadAssets,
    ProviderCapability::AnimatedStickers,
];

#[must_use]
pub fn all_provider_metadata() -> Vec<ProviderMetadata> {
    vec![
        ProviderMetadata {
            id: "telegram",
            display_name: "Telegram",
            capabilities: TELEGRAM_CAPABILITIES,
            status: ProviderStatus::Implemented,
        },
        ProviderMetadata {
            id: "line-stickers",
            display_name: "LINE Stickers",
            capabilities: LINE_STICKER_CAPABILITIES,
            status: ProviderStatus::Implemented,
        },
        ProviderMetadata {
            id: "line-emojis",
            display_name: "LINE Emojis",
            capabilities: LINE_EMOJI_CAPABILITIES,
            status: ProviderStatus::Implemented,
        },
        planned("signal", "Signal"),
        planned("whatsapp", "WhatsApp"),
        planned("kakao", "Kakao"),
        planned("band", "Band"),
        planned("ogq", "OGQ"),
        planned("viber", "Viber"),
    ]
}

fn planned(id: &'static str, display_name: &'static str) -> ProviderMetadata {
    ProviderMetadata {
        id,
        display_name,
        capabilities: PLANNED_CAPABILITIES,
        status: ProviderStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use crate::registry::{all_provider_metadata, ProviderStatus};

    #[test]
    fn registry_contains_implemented_and_planned_providers() {
        let providers = all_provider_metadata();

        assert!(providers
            .iter()
            .any(|provider| provider.id == "telegram"
                && provider.status == ProviderStatus::Implemented));
        assert!(providers
            .iter()
            .any(|provider| provider.id == "line-stickers"
                && provider.status == ProviderStatus::Implemented));
        assert!(providers.iter().any(|provider| provider.id == "line-emojis"
            && provider.status == ProviderStatus::Implemented));
        assert!(providers
            .iter()
            .any(|provider| provider.id == "signal" && provider.status == ProviderStatus::Planned));
        assert!(providers.iter().any(
            |provider| provider.id == "whatsapp" && provider.status == ProviderStatus::Planned
        ));
        assert!(providers
            .iter()
            .any(|provider| provider.id == "kakao" && provider.status == ProviderStatus::Planned));
        assert!(providers
            .iter()
            .any(|provider| provider.id == "band" && provider.status == ProviderStatus::Planned));
        assert!(providers
            .iter()
            .any(|provider| provider.id == "ogq" && provider.status == ProviderStatus::Planned));
        assert!(providers
            .iter()
            .any(|provider| provider.id == "viber" && provider.status == ProviderStatus::Planned));
    }
}

#![doc = "Telegram Bot API client boundary for MoreStickersManager-rs."]

pub mod bot;
pub mod publish;

pub use bot::{TelegramBotConfig, TelegramBotError, TelegramBotToken};
pub use publish::{
    apply_sticker_set_mutations, fetch_sticker_set, publish_sticker_set, TelegramFetchedSticker,
    TelegramFetchedStickerSet, TelegramPublishError, TelegramPublishRequest,
    TelegramPublishSticker, TelegramPublishedSet, TelegramStickerSetApi,
    TelegramStickerSetMutation, TeloxideTelegramStickerSetApi,
};

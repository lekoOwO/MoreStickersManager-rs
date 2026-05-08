#![doc = "Telegram Bot API client boundary for MoreStickersManager-rs."]

pub mod bot;
pub mod publish;

pub use bot::{TelegramBotConfig, TelegramBotError, TelegramBotToken};
pub use publish::{
    apply_sticker_set_mutations, publish_sticker_set, TelegramPublishError, TelegramPublishRequest,
    TelegramPublishSticker, TelegramPublishedSet, TelegramStickerSetApi,
    TelegramStickerSetMutation, TeloxideTelegramStickerSetApi,
};

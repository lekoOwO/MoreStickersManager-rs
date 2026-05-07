use async_trait::async_trait;
use teloxide::{
    payloads::CreateNewStickerSetSetters,
    requests::{Request, Requester},
    types::{InputSticker, StickerType, UserId},
    Bot,
};

/// One sticker prepared for Telegram publication.
#[derive(Debug)]
pub struct TelegramPublishSticker {
    /// Source MSM sticker ID for logs and test assertions.
    pub source_sticker_id: String,
    /// Teloxide input sticker with prepared media.
    pub input: InputSticker,
}

/// Request to create a Telegram sticker set and append remaining stickers.
#[derive(Debug)]
pub struct TelegramPublishRequest {
    /// Telegram user ID that owns the sticker set.
    pub owner_user_id: i64,
    /// Telegram sticker set name.
    pub sticker_set_name: String,
    /// Telegram sticker set title.
    pub title: String,
    /// Telegram sticker set type.
    pub sticker_type: StickerType,
    /// Stickers used for `createNewStickerSet`.
    pub initial_stickers: Vec<TelegramPublishSticker>,
    /// Stickers appended after creation.
    pub append_stickers: Vec<TelegramPublishSticker>,
}

/// Published Telegram sticker set metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelegramPublishedSet {
    /// Telegram sticker set name.
    pub sticker_set_name: String,
    /// Telegram sticker set title.
    pub title: String,
    /// Total published sticker count.
    pub sticker_count: usize,
    /// Public Telegram add-stickers URL.
    pub url: String,
}

/// Low-level Telegram sticker set API boundary.
#[async_trait]
pub trait TelegramStickerSetApi: Send + Sync {
    /// Creates the sticker set with the initial sticker batch.
    async fn create_new_sticker_set(
        &self,
        owner_user_id: i64,
        sticker_set_name: &str,
        title: &str,
        sticker_type: StickerType,
        stickers: Vec<TelegramPublishSticker>,
    ) -> Result<(), TelegramPublishError>;

    /// Appends one sticker to an existing sticker set.
    async fn add_sticker_to_set(
        &self,
        owner_user_id: i64,
        sticker_set_name: &str,
        sticker: TelegramPublishSticker,
    ) -> Result<(), TelegramPublishError>;
}

/// Errors raised while publishing Telegram sticker sets.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum TelegramPublishError {
    /// Publication request contains no initial stickers.
    #[error("Telegram publication requires at least one initial sticker")]
    EmptyInitialStickers,

    /// Telegram owner user ID cannot be represented by teloxide.
    #[error("invalid Telegram owner user ID: {owner_user_id}")]
    InvalidOwnerUserId {
        /// Invalid user ID.
        owner_user_id: i64,
    },

    /// Telegram API request failed.
    #[error("Telegram API error: {message}")]
    Api {
        /// Failure detail.
        message: String,
    },
}

/// `teloxide` implementation of the Telegram sticker set API boundary.
#[derive(Clone, Debug)]
pub struct TeloxideTelegramStickerSetApi {
    bot: Bot,
}

impl TeloxideTelegramStickerSetApi {
    /// Creates a new adapter from a configured teloxide bot.
    #[must_use]
    pub const fn new(bot: Bot) -> Self {
        Self { bot }
    }
}

#[async_trait]
impl TelegramStickerSetApi for TeloxideTelegramStickerSetApi {
    async fn create_new_sticker_set(
        &self,
        owner_user_id: i64,
        sticker_set_name: &str,
        title: &str,
        sticker_type: StickerType,
        stickers: Vec<TelegramPublishSticker>,
    ) -> Result<(), TelegramPublishError> {
        let stickers = stickers.into_iter().map(|sticker| sticker.input);
        self.bot
            .create_new_sticker_set(
                telegram_user_id(owner_user_id)?,
                sticker_set_name.to_owned(),
                title.to_owned(),
                stickers,
            )
            .sticker_type(sticker_type)
            .send()
            .await
            .map_err(|error| TelegramPublishError::Api {
                message: error.to_string(),
            })?;
        Ok(())
    }

    async fn add_sticker_to_set(
        &self,
        owner_user_id: i64,
        sticker_set_name: &str,
        sticker: TelegramPublishSticker,
    ) -> Result<(), TelegramPublishError> {
        self.bot
            .add_sticker_to_set(
                telegram_user_id(owner_user_id)?,
                sticker_set_name.to_owned(),
                sticker.input,
            )
            .send()
            .await
            .map_err(|error| TelegramPublishError::Api {
                message: error.to_string(),
            })?;
        Ok(())
    }
}

/// Publishes a planned Telegram sticker set through a mockable API boundary.
///
/// # Errors
///
/// Returns an error when the request has no initial stickers or Telegram API calls fail.
pub async fn publish_sticker_set(
    api: &dyn TelegramStickerSetApi,
    request: TelegramPublishRequest,
) -> Result<TelegramPublishedSet, TelegramPublishError> {
    if request.initial_stickers.is_empty() {
        return Err(TelegramPublishError::EmptyInitialStickers);
    }

    let initial_count = request.initial_stickers.len();
    let append_count = request.append_stickers.len();
    api.create_new_sticker_set(
        request.owner_user_id,
        &request.sticker_set_name,
        &request.title,
        request.sticker_type,
        request.initial_stickers,
    )
    .await?;

    for sticker in request.append_stickers {
        api.add_sticker_to_set(request.owner_user_id, &request.sticker_set_name, sticker)
            .await?;
    }

    Ok(TelegramPublishedSet {
        url: format!("https://t.me/addstickers/{}", request.sticker_set_name),
        sticker_set_name: request.sticker_set_name,
        title: request.title,
        sticker_count: initial_count + append_count,
    })
}

fn telegram_user_id(owner_user_id: i64) -> Result<UserId, TelegramPublishError> {
    u64::try_from(owner_user_id)
        .map(UserId)
        .map_err(|_| TelegramPublishError::InvalidOwnerUserId { owner_user_id })
}

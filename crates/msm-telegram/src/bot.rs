use std::fmt;

use teloxide::Bot;
use url::Url;

/// Redacted Telegram bot token wrapper.
#[derive(Clone, Eq, PartialEq)]
pub struct TelegramBotToken {
    secret: String,
}

impl TelegramBotToken {
    /// Creates a token wrapper.
    ///
    /// # Errors
    ///
    /// Returns [`TelegramBotError::InvalidToken`] when the token is empty.
    pub fn new(secret: impl Into<String>) -> Result<Self, TelegramBotError> {
        let secret = secret.into();
        if secret.trim().is_empty() {
            return Err(TelegramBotError::InvalidToken);
        }
        Ok(Self { secret })
    }

    /// Exposes the token only at the teloxide construction boundary.
    #[must_use]
    pub fn as_secret(&self) -> &str {
        &self.secret
    }
}

impl fmt::Debug for TelegramBotToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("TelegramBotToken(<redacted>)")
    }
}

impl fmt::Display for TelegramBotToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<redacted telegram bot token>")
    }
}

/// MSM-owned Telegram bot configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelegramBotConfig {
    token: TelegramBotToken,
    api_url: Url,
}

impl TelegramBotConfig {
    /// Creates a config using Telegram's default Bot API URL.
    ///
    /// # Errors
    ///
    /// Returns [`TelegramBotError::InvalidToken`] when the token is empty.
    pub fn new(token: impl Into<String>) -> Result<Self, TelegramBotError> {
        Ok(Self {
            token: TelegramBotToken::new(token)?,
            api_url: default_api_url(),
        })
    }

    /// Replaces the Bot API URL for self-hosted Bot API servers or future tests.
    ///
    /// # Errors
    ///
    /// Returns [`TelegramBotError::InvalidApiUrl`] when the URL is invalid.
    pub fn with_api_url(mut self, api_url: impl AsRef<str>) -> Result<Self, TelegramBotError> {
        self.api_url =
            Url::parse(api_url.as_ref()).map_err(|error| TelegramBotError::InvalidApiUrl {
                message: error.to_string(),
            })?;
        Ok(self)
    }

    /// Returns the configured Bot API URL.
    #[must_use]
    pub const fn api_url(&self) -> &Url {
        &self.api_url
    }

    /// Returns the redacted token wrapper.
    #[must_use]
    pub const fn token(&self) -> &TelegramBotToken {
        &self.token
    }

    /// Builds a teloxide bot using the configured token.
    ///
    /// Teloxide owns Bot API request execution. MSM keeps this method as the
    /// single boundary where the raw token leaves the redacted wrapper.
    pub fn build_bot(&self) -> Bot {
        Bot::new(self.token.as_secret()).set_api_url(self.api_url.clone())
    }
}

/// Errors raised while building Telegram bot configuration.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum TelegramBotError {
    /// Empty bot token.
    #[error("invalid Telegram bot token")]
    InvalidToken,

    /// Invalid Bot API URL.
    #[error("invalid Telegram Bot API URL: {message}")]
    InvalidApiUrl {
        /// URL parser failure.
        message: String,
    },
}

fn default_api_url() -> Url {
    Url::parse("https://api.telegram.org").expect("static Telegram API URL is valid")
}

use msm_telegram::{TelegramBotConfig, TelegramBotError, TelegramBotToken};
use teloxide::Bot;

#[test]
fn token_debug_display_and_config_debug_are_redacted() {
    let token = TelegramBotToken::new("123456:very-secret").unwrap();
    assert!(!format!("{token:?}").contains("very-secret"));
    assert!(!token.to_string().contains("very-secret"));

    let config = TelegramBotConfig::new("123456:very-secret").unwrap();
    assert!(!format!("{config:?}").contains("very-secret"));
}

#[test]
fn empty_tokens_are_rejected() {
    assert_eq!(
        TelegramBotToken::new("   ").unwrap_err(),
        TelegramBotError::InvalidToken
    );
    assert_eq!(
        TelegramBotConfig::new("").unwrap_err(),
        TelegramBotError::InvalidToken
    );
}

#[test]
fn bot_api_url_can_be_configured_and_validated() {
    let config = TelegramBotConfig::new("123456:secret")
        .unwrap()
        .with_api_url("http://127.0.0.1:8081")
        .unwrap();

    assert_eq!(config.api_url().as_str(), "http://127.0.0.1:8081/");
    assert!(matches!(
        TelegramBotConfig::new("123456:secret")
            .unwrap()
            .with_api_url("not a url")
            .unwrap_err(),
        TelegramBotError::InvalidApiUrl { .. }
    ));
}

#[test]
fn config_builds_a_teloxide_bot() {
    let bot = TelegramBotConfig::new("123456:secret")
        .unwrap()
        .with_api_url("http://127.0.0.1:8081")
        .unwrap()
        .build_bot();

    assert_type::<Bot>(&bot);
    assert_eq!(bot.api_url().as_str(), "http://127.0.0.1:8081/");
}

fn assert_type<T>(_value: &T) {}

use crate::{DomainError, DomainResult};

fn validate_component(component: &str) -> DomainResult<()> {
    if component.is_empty() {
        return Err(DomainError::InvalidProviderIdComponent {
            component: component.to_owned(),
            reason: "component must not be empty",
        });
    }

    if component.contains(':') {
        return Err(DomainError::InvalidProviderIdComponent {
            component: component.to_owned(),
            reason: "component must not contain ':'",
        });
    }

    Ok(())
}

/// Builds a MoreStickers-compatible Telegram sticker pack ID.
///
/// # Errors
///
/// Returns an error when `sticker_set_name` is empty or contains `:`.
pub fn telegram_pack_id(sticker_set_name: &str) -> DomainResult<String> {
    validate_component(sticker_set_name)?;
    Ok(format!("MoreStickers:Telegram:Pack:{sticker_set_name}"))
}

/// Builds a MoreStickers-compatible Telegram sticker ID.
///
/// # Errors
///
/// Returns an error when `sticker_set_name` or `file_unique_id` is empty or contains `:`.
pub fn telegram_sticker_id(sticker_set_name: &str, file_unique_id: &str) -> DomainResult<String> {
    validate_component(sticker_set_name)?;
    validate_component(file_unique_id)?;
    Ok(format!(
        "MoreStickers:Telegram:Sticker:{sticker_set_name}:{file_unique_id}"
    ))
}

/// Builds a MoreStickers-compatible LINE sticker pack ID.
///
/// # Errors
///
/// Returns an error when `pack_id` is empty or contains `:`.
pub fn line_sticker_pack_id(pack_id: &str) -> DomainResult<String> {
    validate_component(pack_id)?;
    Ok(format!("MoreStickers:Line:Pack:{pack_id}"))
}

/// Builds a MoreStickers-compatible LINE sticker ID.
///
/// # Errors
///
/// Returns an error when `pack_id` or `sticker_id` is empty or contains `:`.
pub fn line_sticker_id(pack_id: &str, sticker_id: &str) -> DomainResult<String> {
    validate_component(pack_id)?;
    validate_component(sticker_id)?;
    Ok(format!("MoreStickers:Line:Sticker:{pack_id}:{sticker_id}"))
}

/// Builds a MoreStickers-compatible LINE emoji pack ID.
///
/// # Errors
///
/// Returns an error when `pack_id` is empty or contains `:`.
pub fn line_emoji_pack_id(pack_id: &str) -> DomainResult<String> {
    validate_component(pack_id)?;
    Ok(format!("MoreStickers:Line:Emoji-Pack:{pack_id}"))
}

/// Builds a MoreStickers-compatible LINE emoji ID.
///
/// # Errors
///
/// Returns an error when `pack_id` or `emoji_id` is empty or contains `:`.
pub fn line_emoji_id(pack_id: &str, emoji_id: &str) -> DomainResult<String> {
    validate_component(pack_id)?;
    validate_component(emoji_id)?;
    Ok(format!("MoreStickers:Line-Emoji:{pack_id}:{emoji_id}"))
}

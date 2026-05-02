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

pub fn telegram_pack_id(sticker_set_name: &str) -> DomainResult<String> {
    validate_component(sticker_set_name)?;
    Ok(format!("MoreStickers:Telegram:Pack:{sticker_set_name}"))
}

pub fn telegram_sticker_id(sticker_set_name: &str, file_unique_id: &str) -> DomainResult<String> {
    validate_component(sticker_set_name)?;
    validate_component(file_unique_id)?;
    Ok(format!(
        "MoreStickers:Telegram:Sticker:{sticker_set_name}:{file_unique_id}"
    ))
}

pub fn line_sticker_pack_id(pack_id: &str) -> DomainResult<String> {
    validate_component(pack_id)?;
    Ok(format!("MoreStickers:Line:Pack:{pack_id}"))
}

pub fn line_sticker_id(pack_id: &str, sticker_id: &str) -> DomainResult<String> {
    validate_component(pack_id)?;
    validate_component(sticker_id)?;
    Ok(format!("MoreStickers:Line:Sticker:{pack_id}:{sticker_id}"))
}

pub fn line_emoji_pack_id(pack_id: &str) -> DomainResult<String> {
    validate_component(pack_id)?;
    Ok(format!("MoreStickers:Line:Emoji-Pack:{pack_id}"))
}

pub fn line_emoji_id(pack_id: &str, emoji_id: &str) -> DomainResult<String> {
    validate_component(pack_id)?;
    validate_component(emoji_id)?;
    Ok(format!("MoreStickers:Line-Emoji:{pack_id}:{emoji_id}"))
}

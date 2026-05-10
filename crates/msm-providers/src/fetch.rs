use crate::{ProviderError, ProviderResult};

/// HTTP request shape used to describe provider-side fetch work without
/// performing network I/O inside the normalization crate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderHttpRequestPlan {
    pub method: String,
    pub url: String,
    pub redacted_headers: Vec<(String, String)>,
}

/// How a provider's sticker assets should be discovered and downloaded after
/// metadata has been fetched.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProviderAssetDownloadStrategy {
    /// Telegram metadata contains file IDs; callers must resolve each through
    /// `getFile`, then download from `/file/bot<token>/<file_path>`.
    TelegramBotFileApi,
    /// Provider metadata already contains directly downloadable asset URLs.
    DirectRemoteUrls,
}

/// Provider-side remote fetch boundary. Runtime crates can execute this plan,
/// feed the resulting JSON to `StickerProvider`, then internalize assets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderRemoteFetchPlan {
    pub provider_id: String,
    pub remote_id: String,
    pub metadata_request: ProviderHttpRequestPlan,
    pub asset_strategy: ProviderAssetDownloadStrategy,
}

/// Builds a Telegram sticker-set fetch plan. The bot token is intentionally not
/// embedded; runtime callers should attach it at execution time.
///
/// # Errors
///
/// Returns an error when the Bot API base URL or sticker set name is empty.
pub fn telegram_sticker_set_fetch_plan(
    bot_api_base_url: &str,
    sticker_set_name: &str,
) -> ProviderResult<ProviderRemoteFetchPlan> {
    let base_url = normalized_base_url(bot_api_base_url, "Telegram Bot API base URL")?;
    let remote_id = non_empty(sticker_set_name, "Telegram sticker set name")?;

    Ok(ProviderRemoteFetchPlan {
        provider_id: "telegram".to_owned(),
        remote_id: remote_id.to_owned(),
        metadata_request: ProviderHttpRequestPlan {
            method: "GET".to_owned(),
            url: format!(
                "{base_url}/bot<token>/getStickerSet?name={}",
                percent_encode(remote_id)
            ),
            redacted_headers: vec![("Authorization".to_owned(), "Bearer <redacted>".to_owned())],
        },
        asset_strategy: ProviderAssetDownloadStrategy::TelegramBotFileApi,
    })
}

/// Builds a LINE sticker-shop product fetch plan. The returned HTML/JSON must
/// still be parsed by a runtime-side scraper before normalization.
///
/// # Errors
///
/// Returns an error when the LINE store base URL or pack ID is empty.
pub fn line_sticker_pack_fetch_plan(
    store_base_url: &str,
    pack_id: &str,
) -> ProviderResult<ProviderRemoteFetchPlan> {
    let base_url = normalized_base_url(store_base_url, "LINE store base URL")?;
    let remote_id = non_empty(pack_id, "LINE pack ID")?;

    Ok(ProviderRemoteFetchPlan {
        provider_id: "line-stickers".to_owned(),
        remote_id: remote_id.to_owned(),
        metadata_request: ProviderHttpRequestPlan {
            method: "GET".to_owned(),
            url: format!(
                "{base_url}/stickershop/product/{}/en",
                percent_encode(remote_id)
            ),
            redacted_headers: Vec::new(),
        },
        asset_strategy: ProviderAssetDownloadStrategy::DirectRemoteUrls,
    })
}

fn normalized_base_url<'a>(value: &'a str, label: &str) -> ProviderResult<&'a str> {
    let trimmed = value.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err(ProviderError::InvalidPayload(format!(
            "{label} must not be empty"
        )));
    }
    Ok(trimmed)
}

fn non_empty<'a>(value: &'a str, label: &str) -> ProviderResult<&'a str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ProviderError::InvalidPayload(format!(
            "{label} must not be empty"
        )));
    }
    Ok(trimmed)
}

fn percent_encode(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![byte as char]
            }
            _ => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        line_sticker_pack_fetch_plan, telegram_sticker_set_fetch_plan,
        ProviderAssetDownloadStrategy,
    };

    #[test]
    fn plans_telegram_sticker_set_fetch_without_leaking_bot_token() {
        let plan = telegram_sticker_set_fetch_plan("https://api.telegram.org/", "cat pack")
            .expect("telegram plan");

        assert_eq!(plan.provider_id, "telegram");
        assert_eq!(plan.remote_id, "cat pack");
        assert_eq!(plan.metadata_request.method, "GET");
        assert_eq!(
            plan.metadata_request.url,
            "https://api.telegram.org/bot<token>/getStickerSet?name=cat%20pack"
        );
        assert_eq!(
            plan.metadata_request.redacted_headers,
            vec![("Authorization".to_owned(), "Bearer <redacted>".to_owned())]
        );
        assert_eq!(
            plan.asset_strategy,
            ProviderAssetDownloadStrategy::TelegramBotFileApi
        );
    }

    #[test]
    fn plans_line_sticker_pack_fetch_with_direct_asset_strategy() {
        let plan =
            line_sticker_pack_fetch_plan("https://store.line.me/", "line cats").expect("line plan");

        assert_eq!(plan.provider_id, "line-stickers");
        assert_eq!(
            plan.metadata_request.url,
            "https://store.line.me/stickershop/product/line%20cats/en"
        );
        assert_eq!(
            plan.asset_strategy,
            ProviderAssetDownloadStrategy::DirectRemoteUrls
        );
    }

    #[test]
    fn rejects_empty_remote_fetch_inputs() {
        let error = telegram_sticker_set_fetch_plan(" ", "cat_pack").expect_err("empty base URL");
        assert!(error.to_string().contains("base URL"));

        let error =
            line_sticker_pack_fetch_plan("https://store.line.me", " ").expect_err("empty pack ID");
        assert!(error.to_string().contains("pack ID"));
    }
}

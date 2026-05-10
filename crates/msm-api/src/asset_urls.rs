use axum::http::HeaderMap;
use msm_domain::{resolve_asset_url, AssetUrlConfig, AssetUrlInput, Sticker, StickerPack};

use crate::{ApiError, ApiResult, ApiState};

pub(crate) async fn pack_with_resolved_asset_urls(
    state: &ApiState,
    headers: &HeaderMap,
    tenant_id: &str,
    pack_public_id: &str,
    pack: StickerPack,
) -> ApiResult<StickerPack> {
    let mut config = AssetUrlConfig::new(&public_base_url(headers))?;
    let public_asset_url = state
        .repository()
        .find_tenant(tenant_id)
        .await?
        .and_then(|tenant| tenant.public_asset_url)
        .or_else(|| state.public_asset_url().map(str::to_owned));
    if let Some(public_asset_url) = public_asset_url {
        config = config.with_public_asset_url(&public_asset_url)?;
    }

    let mut pack = pack;
    rewrite_sticker_asset_url(&config, pack_public_id, &mut pack.logo)?;
    for sticker in &mut pack.stickers {
        rewrite_sticker_asset_url(&config, pack_public_id, sticker)?;
    }
    Ok(pack)
}

pub(crate) fn public_base_url(headers: &HeaderMap) -> String {
    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("http");
    let host = headers
        .get("host")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("127.0.0.1:3000");
    format!("{scheme}://{}", host.trim_end_matches('/'))
}

fn rewrite_sticker_asset_url(
    config: &AssetUrlConfig,
    pack_public_id: &str,
    sticker: &mut Sticker,
) -> ApiResult<()> {
    let Some(filename) = sticker.filename.as_deref() else {
        return Ok(());
    };
    sticker.image = resolve_asset_url(
        config,
        &AssetUrlInput {
            pack_public_id,
            filename,
        },
    )
    .map_err(ApiError::from)?;
    Ok(())
}

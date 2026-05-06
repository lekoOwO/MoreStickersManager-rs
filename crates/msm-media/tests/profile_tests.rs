use msm_media::{
    ConversionPlan, MediaKind, MediaPlanError, PreparedMediaSpec, StickerTargetProfile,
};

#[test]
fn static_images_map_to_telegram_static_sticker_profile() {
    let plan = ConversionPlan::for_telegram_regular_sticker(MediaKind::StaticImage).unwrap();

    assert_eq!(plan.source_kind(), MediaKind::StaticImage);
    assert_eq!(
        plan.profile(),
        StickerTargetProfile::telegram_static_sticker()
    );
    assert_eq!(
        plan.prepared_media(),
        &PreparedMediaSpec::new(
            StickerTargetProfile::telegram_static_sticker(),
            "image/png",
            "png",
        )
    );
}

#[test]
fn videos_map_to_telegram_video_sticker_profile() {
    let plan = ConversionPlan::for_telegram_regular_sticker(MediaKind::Video).unwrap();

    assert_eq!(plan.source_kind(), MediaKind::Video);
    assert_eq!(plan.profile().profile_key(), "telegram.sticker.video.v1");
    assert_eq!(plan.prepared_media().mime_type(), "video/webm");
    assert_eq!(plan.prepared_media().extension(), "webm");
}

#[test]
fn unsupported_media_returns_typed_error() {
    let error =
        ConversionPlan::for_telegram_regular_sticker(MediaKind::Unsupported("model/gltf".into()))
            .unwrap_err();

    assert_eq!(
        error,
        MediaPlanError::UnsupportedSource {
            target_profile: "telegram.sticker.regular".into(),
            source_kind: MediaKind::Unsupported("model/gltf".into()),
        }
    );
}

#[test]
fn telegram_profile_keys_are_stable() {
    assert_eq!(
        StickerTargetProfile::telegram_static_sticker().profile_key(),
        "telegram.sticker.static.v1"
    );
    assert_eq!(
        StickerTargetProfile::telegram_video_sticker().profile_key(),
        "telegram.sticker.video.v1"
    );
}

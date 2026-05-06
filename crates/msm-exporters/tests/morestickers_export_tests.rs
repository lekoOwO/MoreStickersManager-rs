use msm_domain::{Sticker, StickerPack};
use msm_exporters::{ExportRequest, ExportTarget, ExportTargetKind, MoreStickersExportTarget};

#[test]
fn morestickers_export_matches_domain_stickerpack_serialization() {
    let pack = sample_pack();
    let expected = pack.to_pretty_json().unwrap();

    let artifact = MoreStickersExportTarget.export_pack(&pack).unwrap();

    assert_eq!(artifact.file_name, "sample.stickerpack");
    assert_eq!(artifact.mime_type, "application/json");
    assert_eq!(artifact.contents, expected.into_bytes());
}

#[test]
fn morestickers_target_exposes_capabilities() {
    let target = MoreStickersExportTarget;
    let capabilities = target.capabilities();

    assert_eq!(capabilities.kind, ExportTargetKind::new("morestickers"));
    assert_eq!(capabilities.display_name, "MoreStickers");
    assert!(!capabilities.supports_remote_publication);
    assert!(!capabilities.supports_media_conversion);
    assert!(!capabilities.requires_credentials);
}

#[test]
fn morestickers_target_plans_serialization_step() {
    let target = MoreStickersExportTarget;
    let plan = target
        .plan(ExportRequest {
            pack_id: "pack_1".into(),
            target_id: "target_morestickers".into(),
            options_json: "{}".into(),
        })
        .unwrap();

    assert_eq!(plan.target_kind, ExportTargetKind::new("morestickers"));
    assert_eq!(plan.pack_id, "pack_1");
    assert_eq!(plan.steps, vec!["serialize .stickerpack"]);
}

fn sample_pack() -> StickerPack {
    let sticker = Sticker {
        id: "MoreStickers:Telegram:Sticker:sample:file".to_owned(),
        image: "https://msm.example/assets/packs/sample/file.webp".to_owned(),
        title: "file".to_owned(),
        sticker_pack_id: "MoreStickers:Telegram:Pack:sample".to_owned(),
        filename: Some("file.webp".to_owned()),
        is_animated: Some(false),
    };

    StickerPack {
        id: "MoreStickers:Telegram:Pack:sample".to_owned(),
        title: "Sample".to_owned(),
        author: None,
        logo: sticker.clone(),
        stickers: vec![sticker],
    }
}

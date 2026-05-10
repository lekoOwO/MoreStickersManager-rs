use msm_media::{MediaProbeReport, PreparedMediaSpec, StickerTargetProfile};

#[test]
fn validates_prepared_static_telegram_media_against_target_profile() {
    let report = MediaProbeReport::from_ffprobe_json(
        r#"{
            "streams": [{"codec_type":"video","codec_name":"png","width":512,"height":512}],
            "format": {"format_name":"png_pipe", "size":"2048"}
        }"#,
    )
    .unwrap();
    let spec = PreparedMediaSpec::new(
        StickerTargetProfile::telegram_static_sticker(),
        "image/png",
        "png",
    );

    spec.validate_probe_report(&report).unwrap();
}

#[test]
fn rejects_prepared_media_that_exceeds_target_file_size() {
    let report = MediaProbeReport::from_ffprobe_json(
        r#"{
            "streams": [{"codec_type":"video","codec_name":"png","width":512,"height":512}],
            "format": {"format_name":"png_pipe", "size":"1048576"}
        }"#,
    )
    .unwrap();
    let spec = PreparedMediaSpec::new(
        StickerTargetProfile::telegram_static_sticker(),
        "image/png",
        "png",
    );

    let error = spec.validate_probe_report(&report).unwrap_err().to_string();

    assert!(error.contains("file size"));
    assert!(error.contains("telegram.sticker.static.v1"));
}

#[test]
fn rejects_prepared_media_that_exceeds_target_duration() {
    let report = MediaProbeReport::from_ffprobe_json(
        r#"{
            "streams": [{"codec_type":"video","codec_name":"vp9","width":512,"height":512}],
            "format": {"format_name":"matroska,webm", "duration":"4.200", "size":"200000"}
        }"#,
    )
    .unwrap();
    let spec = PreparedMediaSpec::new(
        StickerTargetProfile::telegram_video_sticker(),
        "video/webm",
        "webm",
    );

    let error = spec.validate_probe_report(&report).unwrap_err().to_string();

    assert!(error.contains("duration"));
    assert!(error.contains("telegram.sticker.video.v1"));
}

#[test]
fn rejects_prepared_media_with_wrong_canvas_size() {
    let report = MediaProbeReport::from_ffprobe_json(
        r#"{
            "streams": [{"codec_type":"video","codec_name":"png","width":512,"height":384}],
            "format": {"format_name":"png_pipe", "size":"2048"}
        }"#,
    )
    .unwrap();
    let spec = PreparedMediaSpec::new(
        StickerTargetProfile::telegram_static_sticker(),
        "image/png",
        "png",
    );

    let error = spec.validate_probe_report(&report).unwrap_err().to_string();

    assert!(error.contains("dimensions"));
    assert!(error.contains("512x512"));
}

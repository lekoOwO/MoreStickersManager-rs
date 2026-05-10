use std::path::PathBuf;
use std::time::Duration;

use msm_media::{MediaKind, MediaProbeCommand, MediaProbeReport, MediaProbeToolchain};

#[test]
fn builds_shell_free_ffprobe_command() {
    let command = MediaProbeCommand::new(
        &MediaProbeToolchain::new(PathBuf::from("ffprobe")),
        PathBuf::from("stickers/cat.webp"),
    );

    assert_eq!(command.executable(), PathBuf::from("ffprobe"));
    assert_eq!(command.input_path(), PathBuf::from("stickers/cat.webp"));
    assert_eq!(command.timeout(), Duration::from_secs(15));
    assert_eq!(
        command.args(),
        &[
            "-v",
            "error",
            "-print_format",
            "json",
            "-show_streams",
            "-show_format",
            "stickers/cat.webp",
        ]
    );
}

#[test]
fn parses_static_image_probe_report() {
    let report = MediaProbeReport::from_ffprobe_json(
        r#"{
            "streams": [{"codec_type":"video","codec_name":"png","width":512,"height":384}],
            "format": {"format_name":"png_pipe", "size":"1200"}
        }"#,
    )
    .unwrap();

    assert_eq!(report.kind(), &MediaKind::StaticImage);
    assert_eq!(report.width(), Some(512));
    assert_eq!(report.height(), Some(384));
    assert_eq!(report.duration_ms(), None);
    assert_eq!(report.size_bytes(), Some(1200));
}

#[test]
fn parses_animated_probe_report() {
    let report = MediaProbeReport::from_ffprobe_json(
        r#"{
            "streams": [{"codec_type":"video","codec_name":"webp","width":512,"height":512,"nb_frames":"24"}],
            "format": {"format_name":"webp", "duration":"1.250", "size":"24000"}
        }"#,
    )
    .unwrap();

    assert_eq!(report.kind(), &MediaKind::AnimatedImage);
    assert_eq!(report.duration_ms(), Some(1250));
    assert_eq!(report.size_bytes(), Some(24000));
}

#[test]
fn parses_video_probe_report() {
    let report = MediaProbeReport::from_ffprobe_json(
        r#"{
            "streams": [{"codec_type":"video","codec_name":"h264","width":1280,"height":720}],
            "format": {"format_name":"mov,mp4,m4a,3gp,3g2,mj2", "duration":"2.500"}
        }"#,
    )
    .unwrap();

    assert_eq!(report.kind(), &MediaKind::Video);
    assert_eq!(report.width(), Some(1280));
    assert_eq!(report.height(), Some(720));
    assert_eq!(report.duration_ms(), Some(2500));
}

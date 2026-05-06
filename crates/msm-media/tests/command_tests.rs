use std::path::{Path, PathBuf};
use std::time::Duration;

use msm_media::{
    ConversionCommand, ConversionPlan, ConverterToolchain, MediaKind, PreparedMediaSpec,
    StickerTargetProfile,
};

#[test]
fn static_image_command_is_shell_free_and_targets_png() {
    let toolchain = ConverterToolchain::new(PathBuf::from("ffmpeg"));
    let plan = ConversionPlan::for_telegram_regular_sticker(MediaKind::StaticImage).unwrap();

    let command = ConversionCommand::for_plan(
        &toolchain,
        &plan,
        Path::new("input/source.png"),
        Path::new("out/sticker.png"),
    );

    assert_eq!(command.executable(), Path::new("ffmpeg"));
    assert_eq!(command.input_path(), Path::new("input/source.png"));
    assert_eq!(command.output_path(), Path::new("out/sticker.png"));
    assert_eq!(command.expected_mime_type(), "image/png");
    assert_eq!(command.timeout(), Duration::from_secs(30));
    assert_eq!(
        command.args(),
        &[
            "-y",
            "-i",
            "input/source.png",
            "-vf",
            "scale=512:512:force_original_aspect_ratio=decrease,pad=512:512:(ow-iw)/2:(oh-ih)/2:color=0x00000000",
            "-frames:v",
            "1",
            "out/sticker.png",
        ]
    );
}

#[test]
fn video_sticker_command_is_shell_free_and_targets_webm() {
    let toolchain = ConverterToolchain::new(PathBuf::from("ffmpeg"));
    let plan = ConversionPlan::for_telegram_regular_sticker(MediaKind::Video).unwrap();

    let command = ConversionCommand::for_plan(
        &toolchain,
        &plan,
        Path::new("input/source.mp4"),
        Path::new("out/sticker.webm"),
    );

    assert_eq!(command.executable(), Path::new("ffmpeg"));
    assert_eq!(command.expected_mime_type(), "video/webm");
    assert_eq!(command.timeout(), Duration::from_mins(2));
    assert_eq!(
        command.args(),
        &[
            "-y",
            "-i",
            "input/source.mp4",
            "-an",
            "-t",
            "3",
            "-vf",
            "scale=512:512:force_original_aspect_ratio=decrease,pad=512:512:(ow-iw)/2:(oh-ih)/2:color=0x00000000,fps=30",
            "-c:v",
            "libvpx-vp9",
            "-b:v",
            "0",
            "-crf",
            "35",
            "out/sticker.webm",
        ]
    );
}

#[test]
fn thumbnail_command_is_shell_free_and_targets_png() {
    let toolchain = ConverterToolchain::new(PathBuf::from("ffmpeg"));
    let thumbnail = PreparedMediaSpec::new(
        StickerTargetProfile::telegram_thumbnail(),
        "image/png",
        "png",
    );

    let command = ConversionCommand::for_prepared_media(
        &toolchain,
        &thumbnail,
        Path::new("input/source.webm"),
        Path::new("out/thumb.png"),
    );

    assert_eq!(command.expected_mime_type(), "image/png");
    assert_eq!(command.timeout(), Duration::from_secs(30));
    assert_eq!(
        command.args(),
        &[
            "-y",
            "-i",
            "input/source.webm",
            "-vf",
            "scale=100:100:force_original_aspect_ratio=decrease,pad=100:100:(ow-iw)/2:(oh-ih)/2:color=0x00000000",
            "-frames:v",
            "1",
            "out/thumb.png",
        ]
    );
}

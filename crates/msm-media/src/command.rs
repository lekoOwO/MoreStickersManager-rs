use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::{ConversionPlan, PreparedMediaSpec};

/// Configured converter executable paths.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConverterToolchain {
    ffmpeg_path: PathBuf,
}

impl ConverterToolchain {
    /// Creates a converter toolchain using the given ffmpeg executable path.
    #[must_use]
    pub const fn new(ffmpeg_path: PathBuf) -> Self {
        Self { ffmpeg_path }
    }

    /// Path to the ffmpeg executable.
    #[must_use]
    pub fn ffmpeg_path(&self) -> &Path {
        self.ffmpeg_path.as_path()
    }
}

/// Shell-free command plan for producing one prepared media output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversionCommand {
    executable: PathBuf,
    args: Vec<String>,
    input_path: PathBuf,
    output_path: PathBuf,
    timeout: Duration,
    expected_mime_type: &'static str,
}

impl ConversionCommand {
    /// Builds a converter command for an existing conversion plan.
    #[must_use]
    pub fn for_plan(
        toolchain: &ConverterToolchain,
        plan: &ConversionPlan,
        input_path: &Path,
        output_path: &Path,
    ) -> Self {
        Self::for_prepared_media(toolchain, plan.prepared_media(), input_path, output_path)
    }

    /// Builds a converter command for a prepared media output specification.
    #[must_use]
    pub fn for_prepared_media(
        toolchain: &ConverterToolchain,
        prepared_media: &PreparedMediaSpec,
        input_path: &Path,
        output_path: &Path,
    ) -> Self {
        let timeout = if prepared_media.mime_type() == "video/webm" {
            Duration::from_mins(2)
        } else {
            Duration::from_secs(30)
        };
        let args = if prepared_media.mime_type() == "video/webm" {
            video_sticker_args(prepared_media, input_path, output_path)
        } else {
            static_image_args(prepared_media, input_path, output_path)
        };

        Self {
            executable: toolchain.ffmpeg_path().to_path_buf(),
            args,
            input_path: input_path.to_path_buf(),
            output_path: output_path.to_path_buf(),
            timeout,
            expected_mime_type: prepared_media.mime_type(),
        }
    }

    /// Converter executable path.
    #[must_use]
    pub fn executable(&self) -> &Path {
        self.executable.as_path()
    }

    /// Shell-free argument vector.
    #[must_use]
    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// Source media path passed to the converter.
    #[must_use]
    pub fn input_path(&self) -> &Path {
        self.input_path.as_path()
    }

    /// Output media path passed to the converter.
    #[must_use]
    pub fn output_path(&self) -> &Path {
        self.output_path.as_path()
    }

    /// Maximum time allowed for the conversion attempt.
    #[must_use]
    pub const fn timeout(&self) -> Duration {
        self.timeout
    }

    /// MIME type expected after successful conversion.
    #[must_use]
    pub const fn expected_mime_type(&self) -> &'static str {
        self.expected_mime_type
    }
}

fn static_image_args(
    prepared_media: &PreparedMediaSpec,
    input_path: &Path,
    output_path: &Path,
) -> Vec<String> {
    let canvas = prepared_media.profile().canvas_size_px();
    vec![
        "-y".to_owned(),
        "-i".to_owned(),
        path_arg(input_path),
        "-vf".to_owned(),
        square_filter(canvas, None),
        "-frames:v".to_owned(),
        "1".to_owned(),
        path_arg(output_path),
    ]
}

fn video_sticker_args(
    prepared_media: &PreparedMediaSpec,
    input_path: &Path,
    output_path: &Path,
) -> Vec<String> {
    let canvas = prepared_media.profile().canvas_size_px();
    vec![
        "-y".to_owned(),
        "-i".to_owned(),
        path_arg(input_path),
        "-an".to_owned(),
        "-t".to_owned(),
        "3".to_owned(),
        "-vf".to_owned(),
        square_filter(canvas, Some(30)),
        "-c:v".to_owned(),
        "libvpx-vp9".to_owned(),
        "-b:v".to_owned(),
        "0".to_owned(),
        "-crf".to_owned(),
        "35".to_owned(),
        path_arg(output_path),
    ]
}

fn square_filter(canvas: u16, fps: Option<u16>) -> String {
    let base = format!(
        "scale={canvas}:{canvas}:force_original_aspect_ratio=decrease,pad={canvas}:{canvas}:(ow-iw)/2:(oh-ih)/2:color=0x00000000"
    );
    fps.map_or(base.clone(), |fps| format!("{base},fps={fps}"))
}

fn path_arg(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

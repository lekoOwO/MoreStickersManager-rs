use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use serde::Deserialize;

use crate::{MediaKind, MediaPlanError, MediaPlanResult};

/// Configured ffprobe executable path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaProbeToolchain {
    ffprobe_path: PathBuf,
}

impl MediaProbeToolchain {
    /// Creates a probe toolchain using the given ffprobe executable path.
    #[must_use]
    pub const fn new(ffprobe_path: PathBuf) -> Self {
        Self { ffprobe_path }
    }

    /// Path to the ffprobe executable.
    #[must_use]
    pub fn ffprobe_path(&self) -> &Path {
        self.ffprobe_path.as_path()
    }
}

/// Shell-free ffprobe command plan for inspecting one source asset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaProbeCommand {
    executable: PathBuf,
    args: Vec<String>,
    input_path: PathBuf,
    timeout: Duration,
}

impl MediaProbeCommand {
    /// Builds an ffprobe command for one source media file.
    #[must_use]
    pub fn new(toolchain: &MediaProbeToolchain, input_path: PathBuf) -> Self {
        let input = input_path.to_string_lossy().into_owned();
        Self {
            executable: toolchain.ffprobe_path().to_path_buf(),
            args: vec![
                "-v".to_owned(),
                "error".to_owned(),
                "-print_format".to_owned(),
                "json".to_owned(),
                "-show_streams".to_owned(),
                "-show_format".to_owned(),
                input,
            ],
            input_path,
            timeout: Duration::from_secs(15),
        }
    }

    /// Probe executable path.
    #[must_use]
    pub fn executable(&self) -> PathBuf {
        self.executable.clone()
    }

    /// Shell-free argument vector.
    #[must_use]
    pub fn args(&self) -> Vec<&str> {
        self.args.iter().map(String::as_str).collect()
    }

    /// Source media path passed to ffprobe.
    #[must_use]
    pub fn input_path(&self) -> PathBuf {
        self.input_path.clone()
    }

    /// Maximum time allowed for probing.
    #[must_use]
    pub const fn timeout(&self) -> Duration {
        self.timeout
    }
}

/// Parsed media facts from ffprobe JSON output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaProbeReport {
    kind: MediaKind,
    width: Option<u32>,
    height: Option<u32>,
    duration_ms: Option<u32>,
    size_bytes: Option<u64>,
    codec_name: Option<String>,
}

impl MediaProbeReport {
    /// Parses ffprobe JSON output into normalized media facts.
    ///
    /// # Errors
    ///
    /// Returns an error when the ffprobe JSON cannot be parsed or has no video stream.
    pub fn from_ffprobe_json(input: &str) -> MediaPlanResult<Self> {
        let raw: FfprobeOutput =
            serde_json::from_str(input).map_err(|error| MediaPlanError::ProbeParse {
                message: error.to_string(),
            })?;
        let stream = raw
            .streams
            .iter()
            .find(|stream| stream.codec_type.as_deref() == Some("video"))
            .ok_or_else(|| MediaPlanError::ProbeParse {
                message: "ffprobe output has no video stream".to_owned(),
            })?;
        let duration_ms = raw
            .format
            .as_ref()
            .and_then(|format| parse_seconds_ms(format.duration.as_deref()));
        let size_bytes = raw
            .format
            .as_ref()
            .and_then(|format| format.size.as_deref())
            .and_then(|size| size.parse::<u64>().ok());
        let codec_name = stream.codec_name.clone();
        let kind = classify_kind(raw.format.as_ref(), stream, duration_ms);

        Ok(Self {
            kind,
            width: stream.width,
            height: stream.height,
            duration_ms,
            size_bytes,
            codec_name,
        })
    }

    #[must_use]
    pub const fn kind(&self) -> &MediaKind {
        &self.kind
    }
    #[must_use]
    pub const fn width(&self) -> Option<u32> {
        self.width
    }
    #[must_use]
    pub const fn height(&self) -> Option<u32> {
        self.height
    }
    #[must_use]
    pub const fn duration_ms(&self) -> Option<u32> {
        self.duration_ms
    }
    #[must_use]
    pub const fn size_bytes(&self) -> Option<u64> {
        self.size_bytes
    }
    #[must_use]
    pub fn codec_name(&self) -> Option<&str> {
        self.codec_name.as_deref()
    }
}

#[derive(Debug, Deserialize)]
struct FfprobeOutput {
    #[serde(default)]
    streams: Vec<FfprobeStream>,
    format: Option<FfprobeFormat>,
}

#[derive(Debug, Deserialize)]
struct FfprobeStream {
    codec_type: Option<String>,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    nb_frames: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FfprobeFormat {
    format_name: Option<String>,
    duration: Option<String>,
    size: Option<String>,
}

fn classify_kind(
    format: Option<&FfprobeFormat>,
    stream: &FfprobeStream,
    duration_ms: Option<u32>,
) -> MediaKind {
    let format_name = format
        .and_then(|format| format.format_name.as_deref())
        .unwrap_or_default();
    let codec_name = stream.codec_name.as_deref().unwrap_or_default();
    if matches!(codec_name, "png" | "mjpeg" | "jpeg" | "webp")
        && !has_multiple_frames(stream)
        && duration_ms.unwrap_or_default() <= 1
    {
        return MediaKind::StaticImage;
    }
    if format_name.contains("gif") || format_name.contains("apng") || codec_name == "webp" {
        return MediaKind::AnimatedImage;
    }
    MediaKind::Video
}

fn has_multiple_frames(stream: &FfprobeStream) -> bool {
    stream
        .nb_frames
        .as_deref()
        .and_then(|frames| frames.parse::<u32>().ok())
        .is_some_and(|frames| frames > 1)
}

fn parse_seconds_ms(value: Option<&str>) -> Option<u32> {
    let seconds = value?.parse::<f64>().ok()?;
    if !seconds.is_finite() || seconds <= 0.0 {
        return None;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    Some((seconds * 1000.0).round() as u32)
}

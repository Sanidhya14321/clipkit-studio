//! ffprobe into a typed summary (spec 8.2): duration, streams, frame rate, variable frame rate,
//! rotation and audio track count. Parsed from ffprobe JSON, never from human-readable output.

use std::ffi::OsStr;
use std::path::Path;
use std::time::Duration;

use ck_core::{CkError, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::run::{check_path, run};

const PROBE_TIMEOUT: Duration = Duration::from_secs(30);

/// What import needs to know about one media file. Times in seconds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    /// Container format names as ffprobe reports them (for example `mov,mp4,m4a,3gp,3g2,mj2`).
    pub format: String,
    pub duration: f64,
    /// First video stream that is not cover art; `None` for audio-only files.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video: Option<VideoStream>,
    /// Every audio stream, in file order.
    #[serde(default)]
    pub audio: Vec<AudioStream>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VideoStream {
    pub index: u32,
    pub codec: String,
    /// Coded size, before rotation.
    pub width: u32,
    pub height: u32,
    /// Average frame rate; 0 when ffprobe does not know it.
    pub fps: f64,
    /// Average and nominal frame rates differ by more than 1 percent.
    pub variable_frame_rate: bool,
    /// Display rotation in degrees, normalized to 0, 90, 180 or 270.
    pub rotation: u32,
    /// Size as displayed, after rotation.
    pub display_width: u32,
    pub display_height: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AudioStream {
    pub index: u32,
    pub codec: String,
    pub sample_rate: u32,
    pub channels: u32,
}

/// Probes `path` with ffprobe.
pub fn probe(path: &Path) -> Result<MediaInfo, CkError> {
    check_path(path)?;
    let args: [&OsStr; 9] = [
        "-hide_banner".as_ref(),
        "-v".as_ref(),
        "error".as_ref(),
        "-show_streams".as_ref(),
        "-show_format".as_ref(),
        "-of".as_ref(),
        "json".as_ref(),
        "-i".as_ref(),
        path.as_os_str(),
    ];
    let out = run("ffprobe", &args, PROBE_TIMEOUT)?;
    parse_probe(&out.stdout).map_err(|e| {
        CkError::new(
            ErrorCode::Media,
            format!("{}: {}", path.display(), e.message),
        )
    })
}

#[derive(Deserialize)]
struct RawProbe {
    #[serde(default)]
    streams: Vec<RawStream>,
    format: Option<RawFormat>,
}

#[derive(Deserialize)]
struct RawFormat {
    #[serde(default)]
    format_name: String,
    duration: Option<String>,
}

#[derive(Deserialize)]
struct RawStream {
    index: u32,
    #[serde(default)]
    codec_type: String,
    #[serde(default)]
    codec_name: String,
    width: Option<u32>,
    height: Option<u32>,
    r_frame_rate: Option<String>,
    avg_frame_rate: Option<String>,
    sample_rate: Option<String>,
    channels: Option<u32>,
    duration: Option<String>,
    #[serde(default)]
    disposition: RawDisposition,
    #[serde(default)]
    tags: RawTags,
    #[serde(default)]
    side_data_list: Vec<RawSideData>,
}

#[derive(Deserialize, Default)]
struct RawDisposition {
    #[serde(default)]
    attached_pic: u8,
}

#[derive(Deserialize, Default)]
struct RawTags {
    rotate: Option<String>,
}

#[derive(Deserialize)]
struct RawSideData {
    rotation: Option<f64>,
}

/// Parses ffprobe `-of json` output. Fails with `E_MEDIA` when the file has no usable
/// video or audio stream or no duration.
pub fn parse_probe(json: &[u8]) -> Result<MediaInfo, CkError> {
    let raw: RawProbe = serde_json::from_slice(json)
        .map_err(|e| CkError::new(ErrorCode::Media, format!("unreadable ffprobe output: {e}")))?;
    let format = raw.format.as_ref();
    let mut duration = format
        .and_then(|f| f.duration.as_deref())
        .and_then(parse_f64)
        .unwrap_or(0.0);
    let video = raw
        .streams
        .iter()
        .find(|s| s.codec_type == "video" && s.disposition.attached_pic == 0)
        .map(video_stream);
    let audio: Vec<AudioStream> = raw
        .streams
        .iter()
        .filter(|s| s.codec_type == "audio")
        .map(|s| AudioStream {
            index: s.index,
            codec: s.codec_name.clone(),
            sample_rate: s
                .sample_rate
                .as_deref()
                .and_then(|r| r.parse().ok())
                .unwrap_or(0),
            channels: s.channels.unwrap_or(0),
        })
        .collect();
    if video.is_none() && audio.is_empty() {
        return Err(CkError::new(ErrorCode::Media, "no video or audio stream"));
    }
    if duration <= 0.0 {
        duration = raw
            .streams
            .iter()
            .filter_map(|s| s.duration.as_deref().and_then(parse_f64))
            .fold(0.0, f64::max);
    }
    if !duration.is_finite() || duration <= 0.0 {
        return Err(CkError::new(ErrorCode::Media, "no duration"));
    }
    Ok(MediaInfo {
        format: format.map(|f| f.format_name.clone()).unwrap_or_default(),
        duration,
        video,
        audio,
    })
}

fn video_stream(s: &RawStream) -> VideoStream {
    let avg = s.avg_frame_rate.as_deref().and_then(parse_rate);
    let nominal = s.r_frame_rate.as_deref().and_then(parse_rate);
    let fps = avg.or(nominal).unwrap_or(0.0);
    let variable_frame_rate = match (avg, nominal) {
        (Some(a), Some(n)) => (a - n).abs() > n * 0.01,
        _ => false,
    };
    let degrees = s
        .side_data_list
        .iter()
        .find_map(|d| d.rotation)
        .or_else(|| s.tags.rotate.as_deref().and_then(parse_f64))
        .unwrap_or(0.0);
    let rotation = normalize_rotation(degrees);
    let (width, height) = (s.width.unwrap_or(0), s.height.unwrap_or(0));
    let (display_width, display_height) = if rotation % 180 == 90 {
        (height, width)
    } else {
        (width, height)
    };
    VideoStream {
        index: s.index,
        codec: s.codec_name.clone(),
        width,
        height,
        fps,
        variable_frame_rate,
        rotation,
        display_width,
        display_height,
    }
}

fn parse_f64(s: &str) -> Option<f64> {
    s.trim().parse::<f64>().ok().filter(|v| v.is_finite())
}

/// `30000/1001` to 29.97; `0/0` and unparsable rates to `None`.
fn parse_rate(s: &str) -> Option<f64> {
    let (n, d) = s.split_once('/').unwrap_or((s, "1"));
    let (n, d) = (parse_f64(n)?, parse_f64(d)?);
    (n > 0.0 && d > 0.0).then(|| n / d)
}

/// Rounds to the nearest quarter turn and maps into 0..360 (ffprobe reports -90 for 270).
fn normalize_rotation(degrees: f64) -> u32 {
    let quarter = (degrees / 90.0).round().rem_euclid(4.0);
    (quarter as u32) * 90
}

#[cfg(test)]
mod tests;

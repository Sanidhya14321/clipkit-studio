//! The media side of import (spec 8.2): content hash, 720p proxy and audio extraction into
//! `.clipkit/cache/`, keyed by hash. The original is only read, never modified.

use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use ck_core::{CkError, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::run::{check_path, run};

/// Proxies are at most this tall; smaller sources keep their size.
pub const PROXY_MAX_HEIGHT: u32 = 720;
/// Sample rate of extracted audio.
pub const AUDIO_SAMPLE_RATE: u32 = 48_000;

const TRANSCODE_TIMEOUT: Duration = Duration::from_secs(30 * 60);

/// Hex SHA-256 of the file's bytes, read in chunks.
pub fn hash_file(path: &Path) -> Result<String, CkError> {
    let io = |e: std::io::Error| CkError::new(ErrorCode::Media, format!("{}: {e}", path.display()));
    let mut file = File::open(path).map_err(io)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = file.read(&mut buf).map_err(io)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

/// Where the derived files of one source live, keyed by its content hash.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CachePaths {
    pub proxy: PathBuf,
    pub audio: PathBuf,
}

impl CachePaths {
    /// Paths under `cache_dir` (normally `.clipkit/cache/`). Refuses a hash that is not hex.
    pub fn new(cache_dir: &Path, hash: &str) -> Result<Self, CkError> {
        if hash.len() < 16 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(CkError::new(
                ErrorCode::Request,
                format!("cache key {hash:?} is not a hex content hash"),
            ));
        }
        Ok(Self {
            proxy: cache_dir.join("proxies").join(format!("{hash}.mp4")),
            audio: cache_dir.join("audio").join(format!("{hash}.wav")),
        })
    }
}

/// Writes an H.264 proxy at most [`PROXY_MAX_HEIGHT`] tall (even size, aspect kept, rotation
/// applied) with AAC audio when the source has any. Skips the work when `out` already exists.
pub fn make_proxy(src: &Path, out: &Path) -> Result<(), CkError> {
    let scale = format!("scale=-2:'min({PROXY_MAX_HEIGHT},trunc(ih/2)*2)'");
    let args: [&OsStr; 20] = [
        "-map".as_ref(),
        "0:v:0".as_ref(),
        "-map".as_ref(),
        "0:a:0?".as_ref(),
        "-vf".as_ref(),
        scale.as_ref(),
        "-c:v".as_ref(),
        "libx264".as_ref(),
        "-preset".as_ref(),
        "veryfast".as_ref(),
        "-crf".as_ref(),
        "23".as_ref(),
        "-pix_fmt".as_ref(),
        "yuv420p".as_ref(),
        "-c:a".as_ref(),
        "aac".as_ref(),
        "-movflags".as_ref(),
        "+faststart".as_ref(),
        "-f".as_ref(),
        "mp4".as_ref(),
    ];
    transcode(src, out, &args)
}

/// Writes the first audio stream as 16-bit PCM WAV at [`AUDIO_SAMPLE_RATE`]. Skips the work when
/// `out` already exists. Fails with `E_MEDIA` when the source has no audio.
pub fn extract_audio(src: &Path, out: &Path) -> Result<(), CkError> {
    let rate = AUDIO_SAMPLE_RATE.to_string();
    let args: [&OsStr; 9] = [
        "-map".as_ref(),
        "0:a:0".as_ref(),
        "-vn".as_ref(),
        "-c:a".as_ref(),
        "pcm_s16le".as_ref(),
        "-ar".as_ref(),
        rate.as_ref(),
        "-f".as_ref(),
        "wav".as_ref(),
    ];
    transcode(src, out, &args)
}

/// Runs `ffmpeg -i src <middle> tmp`, then renames tmp to `out`, so a crash never leaves a
/// half-written cache entry under the final name.
fn transcode(src: &Path, out: &Path, middle: &[&OsStr]) -> Result<(), CkError> {
    check_path(src)?;
    check_path(out)?;
    if out.is_file() {
        return Ok(());
    }
    let io = |e: std::io::Error| CkError::new(ErrorCode::Media, format!("{}: {e}", out.display()));
    let dir = out.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(dir).map_err(io)?;
    let name = out.file_name().unwrap_or(OsStr::new("out"));
    let mut tmp_name = OsStr::new(".tmp-").to_os_string();
    tmp_name.push(name);
    let tmp = dir.join(tmp_name);
    let mut args: Vec<&OsStr> = vec![
        "-hide_banner".as_ref(),
        "-nostdin".as_ref(),
        "-v".as_ref(),
        "error".as_ref(),
        "-y".as_ref(),
        "-i".as_ref(),
        src.as_os_str(),
    ];
    args.extend_from_slice(middle);
    args.push(tmp.as_os_str());
    let result = run("ffmpeg", &args, TRANSCODE_TIMEOUT)
        .and_then(|_| std::fs::rename(&tmp, out).map_err(io));
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

#[cfg(test)]
mod tests;

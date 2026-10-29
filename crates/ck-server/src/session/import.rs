//! `import` (spec 3.2 Source, spec 12 Phase 5): probe a media file, hash it, write its 720p
//! proxy and 48 kHz audio into `.clipkit/cache/`, and register it as a project source in one
//! agent revision. Importing the same content twice returns the existing source unchanged.

use std::path::{Path, PathBuf};

use ck_core::model::{Source, SourceKind};
use ck_core::{CkError, CkResult, ErrorCode};
use ck_media::{CachePaths, MediaInfo};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::Session;

/// Hex digits of the content hash used in a source id (`src-<8 hex>`).
const ID_HASH_LEN: usize = 8;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImportRequest {
    /// Absolute path of the media file.
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    /// Project revision after the write (unchanged when the content was already imported).
    pub rev: u64,
    /// True when a new source was added.
    pub added: bool,
    pub source: Source,
    /// Audio tracks in the file (only the first is extracted).
    pub audio_tracks: usize,
    /// True when the video has a variable frame rate.
    pub vfr: bool,
}

pub fn source_kind(info: &MediaInfo) -> CkResult<SourceKind> {
    match (&info.video, info.audio.is_empty()) {
        (Some(_), _) if info.is_still() => Ok(SourceKind::Image),
        (Some(_), _) => Ok(SourceKind::Video),
        (None, false) => Ok(SourceKind::Audio),
        (None, true) => Err(CkError::new(
            ErrorCode::Media,
            "file has no video or audio stream".to_owned(),
        )),
    }
}

/// `path` relative to `base` with `/` separators when it lies inside, else absolute.
fn stored_path(path: &Path, base: &Path) -> String {
    match path.strip_prefix(base) {
        Ok(rel) => rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/"),
        Err(_) => path.to_string_lossy().into_owned(),
    }
}

impl Session {
    /// `import`: the media work runs before the project lock is taken, so a long proxy encode
    /// never blocks other calls.
    pub fn import(&self, req: &ImportRequest) -> CkResult<ImportResult> {
        let path = PathBuf::from(&req.path);
        if !path.is_absolute() {
            return Err(CkError::new(
                ErrorCode::Request,
                format!("import path {:?} is not absolute", req.path),
            ));
        }
        let info = ck_media::probe(&path)?;
        let kind = source_kind(&info)?;
        let hash = ck_media::hash_file(&path)?;
        let root = self.store.root().to_path_buf();
        let workdir = root
            .parent()
            .map_or_else(|| root.clone(), Path::to_path_buf);
        let cache = CachePaths::new(&root.join("cache"), &hash)?;
        let mut proxy_path = None;
        if kind == SourceKind::Video {
            ck_media::make_proxy(&path, &cache.proxy)?;
            proxy_path = Some(stored_path(&cache.proxy, &workdir));
        }
        if !info.audio.is_empty() {
            ck_media::extract_audio(&path, &cache.audio)?;
        }
        let source = Source {
            id: format!("src-{}", hash.get(..ID_HASH_LEN).unwrap_or(&hash)),
            kind,
            path: stored_path(&path, &workdir),
            proxy_path,
            hash,
            duration_sec: (kind != SourceKind::Image).then_some(info.duration),
            transcript_path: None,
        };
        let audio_tracks = info.audio.len();
        let vfr = info.video.as_ref().is_some_and(|v| v.variable_frame_rate);

        let mut doc = self.doc()?;
        let project = doc.project();
        if let Some(existing) = project.sources.iter().find(|s| s.hash == source.hash) {
            return Ok(ImportResult {
                rev: project.rev,
                added: false,
                source: existing.clone(),
                audio_tracks,
                vfr,
            });
        }
        let ops = json!([{ "op": "add", "path": "/sources/-", "value": source }]);
        let rev = self.commit_agent(&mut doc, ops, format!("import: {}", source.id))?;
        Ok(ImportResult {
            rev: rev.rev,
            added: true,
            source,
            audio_tracks,
            vfr,
        })
    }
}

//! The EDIT base pass (spec 4, spec 8): transcribe a source, read the packed transcript, suggest
//! cuts and apply them as footage segments on the video track.

use std::path::{Path, PathBuf};

use ck_core::model::{Item, ItemBody, Source, Track, TrackKind};
use ck_core::{CkError, CkResult, ErrorCode};
use ck_media::CachePaths;
use ck_voice::{CutCandidate, PackedTranscript, Transcript, kept_ranges, suggest_cuts};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{Session, is_zero};

/// Id of the video track `cut apply` creates when the project has none.
pub const VIDEO_TRACK: &str = "video-1";

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceRequest {
    /// Source id (from `import`).
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TranscribeResult {
    pub rev: u64,
    pub source: String,
    pub words: usize,
    pub transcript_path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptView {
    pub source: String,
    #[serde(flatten)]
    pub packed: PackedTranscript,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CutSuggestResult {
    pub source: String,
    pub duration_sec: f64,
    pub cuts: Vec<CutCandidate>,
    /// Hard scene changes in the footage (second cut signal), seconds.
    #[serde(default)]
    pub scenes: Vec<f64>,
    /// Cuts left out to keep the answer under its size cap.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub omitted: usize,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CutApplyRequest {
    pub source: String,
    /// Cut ids from `cut suggest`; empty applies every suggested cut.
    #[serde(default)]
    pub ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CutApplyResult {
    pub rev: u64,
    pub track: String,
    /// Footage segments now on the track for this source.
    pub segments: Vec<String>,
    pub removed_sec: f64,
    pub duration_sec: f64,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub omitted: usize,
}

fn round3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

impl Session {
    fn workdir(&self) -> PathBuf {
        let root = self.store.root();
        root.parent().unwrap_or(root).to_path_buf()
    }

    fn source(&self, id: &str) -> CkResult<Source> {
        self.doc()?
            .project()
            .sources
            .iter()
            .find(|s| s.id == id)
            .cloned()
            .ok_or_else(|| CkError::new(ErrorCode::NotFound, format!("no source with id {id:?}")))
    }

    fn resolve(&self, stored: &str) -> PathBuf {
        let p = Path::new(stored);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            self.workdir().join(p)
        }
    }

    fn transcript(&self, src: &Source) -> CkResult<Transcript> {
        let path = src.transcript_path.as_deref().ok_or_else(|| {
            CkError::new(
                ErrorCode::NotFound,
                format!("source {:?} has no transcript", src.id),
            )
            .with_hint("Run ck transcribe <source> first.")
        })?;
        ck_voice::load(&self.resolve(path))
    }

    /// `transcribe`: the extracted audio through the transcriber; the word-level transcript is
    /// stored on disk and its path written to the source.
    pub fn transcribe(&self, req: &SourceRequest) -> CkResult<TranscribeResult> {
        let src = self.source(&req.source)?;
        let cache = CachePaths::new(&self.store.root().join("cache"), &src.hash)?;
        if !cache.audio.is_file() {
            return Err(CkError::new(
                ErrorCode::Media,
                format!("source {:?} has no extracted audio", src.id),
            ));
        }
        let transcript = self.transcriber.transcribe(&cache.audio)?;
        let out = self
            .store
            .root()
            .join("cache/transcripts")
            .join(format!("{}.json", src.hash));
        ck_voice::save(&out, &transcript)?;
        let workdir = self.workdir();
        let stored = out.strip_prefix(&workdir).map_or_else(
            |_| out.to_string_lossy().into_owned(),
            |p| p.to_string_lossy().replace('\\', "/"),
        );
        let mut doc = self.doc()?;
        let index = doc
            .project()
            .sources
            .iter()
            .position(|s| s.id == src.id)
            .ok_or_else(|| CkError::new(ErrorCode::NotFound, "source removed meanwhile"))?;
        let ops = json!([{
            "op": "add",
            "path": format!("/sources/{index}/transcriptPath"),
            "value": stored,
        }]);
        let rev = self.commit_agent(&mut doc, ops, format!("transcribe: {}", src.id))?;
        Ok(TranscribeResult {
            rev: rev.rev,
            source: src.id,
            words: transcript.words.len(),
            transcript_path: stored,
        })
    }

    /// `transcript get`: the packed transcript (phrases, silences, fillers).
    pub fn transcript_get(&self, req: &SourceRequest) -> CkResult<TranscriptView> {
        let src = self.source(&req.source)?;
        Ok(TranscriptView {
            source: src.id.clone(),
            packed: ck_voice::pack(&self.transcript(&src)?),
        })
    }

    fn duration(src: &Source) -> CkResult<f64> {
        src.duration_sec.filter(|d| *d > 0.0).ok_or_else(|| {
            CkError::new(
                ErrorCode::Request,
                format!("source {:?} has no duration to cut", src.id),
            )
        })
    }

    /// `cut suggest`: silences, fillers and retakes on word boundaries, plus scene changes.
    pub fn cut_suggest(&self, req: &SourceRequest) -> CkResult<CutSuggestResult> {
        let src = self.source(&req.source)?;
        let duration = Self::duration(&src)?;
        let cuts = suggest_cuts(&self.transcript(&src)?, duration);
        let scenes = match &src.proxy_path {
            Some(p) => ck_media::detect_scenes(&self.resolve(p))?
                .into_iter()
                .map(|s| round3(s.at))
                .collect(),
            None => Vec::new(),
        };
        Ok(CutSuggestResult {
            source: src.id,
            duration_sec: duration,
            cuts,
            scenes,
            omitted: 0,
        })
    }

    /// `cut apply`: the source's footage on the video track becomes the kept ranges, laid end
    /// to end from where its first segment started, as one revision.
    pub fn cut_apply(&self, req: &CutApplyRequest) -> CkResult<CutApplyResult> {
        let src = self.source(&req.source)?;
        let duration = Self::duration(&src)?;
        let all = suggest_cuts(&self.transcript(&src)?, duration);
        let chosen: Vec<CutCandidate> = if req.ids.is_empty() {
            all
        } else {
            if let Some(bad) = req.ids.iter().find(|id| !all.iter().any(|c| &c.id == *id)) {
                return Err(
                    CkError::new(ErrorCode::NotFound, format!("no suggested cut {bad:?}"))
                        .with_hint("Run ck cut suggest <source> for the current ids."),
                );
            }
            all.into_iter()
                .filter(|c| req.ids.contains(&c.id))
                .collect()
        };
        let kept = kept_ranges(&chosen, duration);

        let mut doc = self.doc()?;
        let project = doc.project();
        let track_index = project
            .tracks
            .iter()
            .position(|t| t.kind == TrackKind::Video);
        let mut items: Vec<Item> = track_index
            .map(|i| project.tracks[i].items.clone())
            .unwrap_or_default();
        let is_ours =
            |i: &Item| matches!(&i.body, ItemBody::Footage { src: s, .. } if *s == src.id);
        let start_at = items
            .iter()
            .filter(|i| is_ours(i))
            .filter_map(|i| match i.body {
                ItemBody::Footage { at, .. } => Some(at),
                _ => None,
            })
            .fold(f64::INFINITY, f64::min);
        let start_at = if start_at.is_finite() { start_at } else { 0.0 };
        items.retain(|i| !is_ours(i));
        let mut at = start_at;
        let mut segments = Vec::new();
        for (n, (s, e)) in kept.iter().enumerate() {
            let id = format!("{}-seg-{}", src.id, n + 1);
            items.push(Item {
                id: id.clone(),
                name: format!("Footage {}", n + 1),
                purpose: "edit".into(),
                pass: None,
                asset_ref: None,
                brief_deps: Vec::new(),
                provenance: None,
                rev: 0,
                body: ItemBody::Footage {
                    src: src.id.clone(),
                    src_in: round3(*s),
                    src_out: round3(*e),
                    at: round3(at),
                    speed: 1.0,
                    volume: 1.0,
                },
            });
            at += e - s;
            segments.push(id);
        }
        items.sort_by(|a, b| item_at(a).total_cmp(&item_at(b)));
        let (ops, track) = match track_index {
            Some(i) => (
                json!([{ "op": "replace", "path": format!("/tracks/{i}/items"), "value": items }]),
                project.tracks[i].id.clone(),
            ),
            None => {
                let t = Track {
                    id: VIDEO_TRACK.into(),
                    kind: TrackKind::Video,
                    items,
                };
                (
                    json!([{ "op": "add", "path": "/tracks/0", "value": t }]),
                    VIDEO_TRACK.to_owned(),
                )
            }
        };
        let label = format!("cut: {} ({} cuts)", src.id, chosen.len());
        let rev = self.commit_agent(&mut doc, ops, label)?;
        let removed: f64 = chosen.iter().map(|c| c.end - c.start).sum();
        Ok(CutApplyResult {
            rev: rev.rev,
            track,
            segments,
            removed_sec: round3(removed),
            duration_sec: round3(duration - removed),
            omitted: 0,
        })
    }
}

fn item_at(i: &Item) -> f64 {
    match &i.body {
        ItemBody::Footage { at, .. }
        | ItemBody::Clip { at, .. }
        | ItemBody::Voice { at, .. }
        | ItemBody::Audio { at, .. } => *at,
        ItemBody::Caption { .. } => 0.0,
    }
}

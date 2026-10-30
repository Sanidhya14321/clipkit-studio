//! `export` (spec 8.6): the fast ffmpeg path for footage-only timelines, with platform presets
//! and a loudness gate measured on the written file. Projects with component layers need the
//! full render path (the Phase 9 render worker) and are refused for now.

use std::path::{Component, Path, PathBuf};

use ck_core::model::{ItemBody, Project, TrackKind};
use ck_core::{CkError, CkResult, ErrorCode};
use ck_media::{CachePaths, ExportSpec, Loudness, LoudnessTarget, Piece};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Session;

/// Integrated loudness and true peak every preset normalizes to (DECISIONS.md, step 9).
pub const TARGET: LoudnessTarget = LoudnessTarget {
    integrated: -14.0,
    true_peak: -1.0,
};
/// The loudness gate passes within this many LU of the target.
pub const LOUDNESS_TOLERANCE: f64 = 1.0;
/// Gaps shorter than this between footage segments are closed instead of filled with black.
const GAP_MIN: f64 = 0.01;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Preset {
    /// The project's own frame size.
    #[default]
    Source,
    /// 1920x1080.
    Youtube,
    /// 1080x1920.
    Tiktok,
    /// 1080x1920.
    Reels,
    /// 1080x1920.
    Shorts,
}

impl Preset {
    fn size(self, project: &Project) -> (u32, u32) {
        match self {
            Preset::Source => (project.settings.width, project.settings.height),
            Preset::Youtube => (1920, 1080),
            Preset::Tiktok | Preset::Reels | Preset::Shorts => (1080, 1920),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Preset::Source => "source",
            Preset::Youtube => "youtube",
            Preset::Tiktok => "tiktok",
            Preset::Reels => "reels",
            Preset::Shorts => "shorts",
        }
    }
}

/// The export path (spec 2.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ExportPath {
    /// Footage only: one ffmpeg filter graph.
    Fast,
    /// Component layers: the Remotion render worker, then ck's audio mix.
    Render,
}

/// What `export` writes (spec 8.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ExportFormat {
    /// The video (fast path or render worker).
    #[default]
    Mp4,
    /// OpenTimelineIO timeline.
    Otio,
    /// SRT subtitles from real word timing.
    Srt,
}

fn is_mp4(f: &ExportFormat) -> bool {
    *f == ExportFormat::Mp4
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportRequest {
    /// mp4 (default), otio or srt; otio and srt answer with the timeline export result.
    #[serde(default, skip_serializing_if = "is_mp4")]
    pub format: ExportFormat,
    #[serde(default)]
    pub preset: Preset,
    /// Output path relative to the project folder; default `exports/<project>-<preset>.mp4`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub out: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    /// Written file, relative to the project folder.
    pub path: String,
    pub preset: Preset,
    pub width: u32,
    pub height: u32,
    pub duration_sec: f64,
    pub loudness: Loudness,
    /// True when integrated loudness is within tolerance and true peak at most the target.
    pub loudness_pass: bool,
    /// The video has no audible sound (nothing to normalize; the gate does not apply).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub silent: bool,
    /// Which export path made the file.
    pub via: ExportPath,
    /// License and attribution report written next to the video, when a Source Hub is found.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license_report: Option<String>,
    /// False when the report blocks an asset in the license mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license_ok: Option<bool>,
}

/// A planned footage segment: input index, source in, source out, timeline start.
pub type PlannedSegment = (usize, f64, f64, f64);

/// The fast-path pieces of a footage-only project, in timeline order. Refuses component, voice,
/// audio and caption layers, and footage that overlaps.
pub fn plan(project: &Project) -> CkResult<(Vec<String>, Vec<PlannedSegment>)> {
    let mut sources: Vec<String> = Vec::new();
    let mut segs: Vec<PlannedSegment> = Vec::new();
    for track in &project.tracks {
        for item in &track.items {
            match &item.body {
                ItemBody::Footage {
                    src,
                    src_in,
                    src_out,
                    at,
                    speed,
                    ..
                } if track.kind == TrackKind::Video => {
                    if (*speed - 1.0).abs() > f64::EPSILON {
                        return Err(refuse(&item.id, "footage at another speed"));
                    }
                    let input = match sources.iter().position(|s| s == src) {
                        Some(i) => i,
                        None => {
                            sources.push(src.clone());
                            sources.len() - 1
                        }
                    };
                    segs.push((input, *src_in, *src_out, *at));
                }
                _ => return Err(refuse(&item.id, "a layer the fast path cannot draw")),
            }
        }
    }
    if segs.is_empty() {
        return Err(
            CkError::new(ErrorCode::Request, "nothing to export: no footage")
                .with_hint("Import a video with ck import, or wait for the Phase 9 render path."),
        );
    }
    segs.sort_by(|a, b| a.3.total_cmp(&b.3));
    for pair in segs.windows(2) {
        let end = pair[0].3 + (pair[0].2 - pair[0].1);
        if pair[1].3 < end - GAP_MIN {
            return Err(CkError::new(
                ErrorCode::Request,
                format!("footage overlaps at {:.3}s", pair[1].3),
            ));
        }
    }
    Ok((sources, segs))
}

fn refuse(id: &str, why: &str) -> CkError {
    CkError::new(
        ErrorCode::Request,
        format!("layer {id:?} is {why}; only footage-only projects export for now"),
    )
    .with_hint("Component layers export through the render worker (Phase 9).")
    .with_layers([id.to_owned()])
}

/// A relative path inside the project folder: no root, no `..`.
fn inside(rel: &str) -> CkResult<PathBuf> {
    let p = Path::new(rel);
    let ok = !rel.is_empty()
        && p.components().all(|c| matches!(c, Component::Normal(_)))
        && p.extension().is_some_and(|e| e == "mp4");
    if !ok {
        return Err(CkError::new(
            ErrorCode::Request,
            format!("export path {rel:?} must be a relative .mp4 path inside the project"),
        ));
    }
    Ok(p.to_path_buf())
}

impl Session {
    pub fn export(&self, req: &ExportRequest) -> CkResult<ExportResult> {
        if req.format != ExportFormat::Mp4 {
            return Err(CkError::new(
                ErrorCode::Request,
                "otio and srt are written by /api/export/timeline",
            ));
        }
        let project = self.project()?;
        let workdir = self.workdir();
        let (width, height) = req.preset.size(&project);
        let rel = match &req.out {
            Some(o) => inside(o)?,
            None => {
                PathBuf::from("exports").join(format!("{}-{}.mp4", project.id, req.preset.name()))
            }
        };
        let out = workdir.join(&rel);
        // Spec 2.3: footage-only projects take the fast ffmpeg path; anything with component,
        // voice, audio or caption layers renders through the worker.
        if project.items().next().is_none() {
            return Err(
                CkError::new(ErrorCode::Request, "nothing to export: no layers")
                    .with_hint("Import a video or apply a recipe first."),
            );
        }
        let fast = project
            .items()
            .all(|i| matches!(i.body, ItemBody::Footage { .. }));
        let path = if fast {
            self.export_fast(&project, &out, (width, height))?;
            ExportPath::Fast
        } else {
            self.export_full(&project, &out, req.preset, (width, height))?;
            ExportPath::Render
        };
        self.finish_export(req, &rel, &out, (width, height), path)
    }

    fn export_fast(
        &self,
        project: &Project,
        out: &Path,
        (width, height): (u32, u32),
    ) -> CkResult<()> {
        let (sources, segs) = plan(project)?;
        let mut inputs = Vec::new();
        let mut has_audio = Vec::new();
        for id in &sources {
            let src = project
                .sources
                .iter()
                .find(|s| &s.id == id)
                .ok_or_else(|| CkError::new(ErrorCode::NotFound, format!("no source {id:?}")))?;
            inputs.push(self.resolve(&src.path));
            has_audio.push(
                CachePaths::new(&self.cache_dir(), &src.hash)?
                    .audio
                    .is_file(),
            );
        }
        let mut pieces = Vec::new();
        let mut cursor = 0.0;
        for (input, src_in, src_out, at) in segs {
            if at - cursor > GAP_MIN {
                pieces.push(Piece::Gap { dur: at - cursor });
            }
            pieces.push(Piece::Segment {
                input,
                src_in,
                src_out,
                has_audio: has_audio[input],
            });
            cursor = at + (src_out - src_in);
        }
        let spec = ExportSpec {
            inputs,
            pieces,
            width,
            height,
            fps: f64::from(project.settings.fps),
            loudness: TARGET,
        };
        ck_media::export(&spec, out)
    }

    /// Render path: muted video from the worker (cached), the timeline's audio mixed by ck
    /// (footage sound and voice takes), then muxed at the preset size.
    fn export_full(
        &self,
        project: &Project,
        out: &Path,
        preset: Preset,
        size: (u32, u32),
    ) -> CkResult<()> {
        let video = self.render_video(project)?;
        let mut clips = Vec::new();
        for item in project.items() {
            match &item.body {
                ItemBody::Footage {
                    src,
                    src_in,
                    src_out,
                    at,
                    ..
                } => {
                    let Some(s) = project.sources.iter().find(|s| &s.id == src) else {
                        continue;
                    };
                    if CachePaths::new(&self.cache_dir(), &s.hash)?.audio.is_file() {
                        clips.push(ck_media::AudioClip {
                            path: self.resolve(&s.path),
                            src_in: *src_in,
                            src_out: *src_out,
                            at: *at,
                            gain: 1.0,
                        });
                    }
                }
                ItemBody::Voice { slot_id, at } => {
                    let take = project
                        .voice_slots
                        .iter()
                        .find(|s| &s.id == slot_id)
                        .and_then(|s| {
                            let id = s.active_take.as_ref()?;
                            s.takes.iter().find(|t| &t.id == id)
                        });
                    if let Some(t) = take {
                        clips.push(ck_media::AudioClip {
                            path: self.resolve(&t.path),
                            src_in: 0.0,
                            src_out: t.duration_sec,
                            at: *at,
                            gain: 1.0,
                        });
                    }
                }
                // Music beds and other audio: each range plays back to back from `at`.
                ItemBody::Audio {
                    src,
                    at,
                    ranges,
                    volume,
                    ..
                } => {
                    let Some(s) = project.sources.iter().find(|s| &s.id == src) else {
                        continue;
                    };
                    let mut t = *at;
                    for r in ranges {
                        clips.push(ck_media::AudioClip {
                            path: self.resolve(&s.path),
                            src_in: r.start,
                            src_out: r.end,
                            at: t,
                            gain: *volume,
                        });
                        t += r.end - r.start;
                    }
                }
                _ => {}
            }
        }
        let duration = ck_core::timeline::duration(project);
        let dir = out.parent().unwrap_or(Path::new("."));
        std::fs::create_dir_all(dir)
            .map_err(|e| CkError::new(ErrorCode::Storage, e.to_string()))?;
        let wav = dir.join(".tmp-mix.wav");
        let mixed = ck_media::mix_audio(&clips, duration, TARGET, &wav).and_then(|()| {
            let scale = (preset != Preset::Source).then_some(size);
            ck_media::mux(&video, &wav, out, scale)
        });
        let _ = std::fs::remove_file(&wav);
        mixed
    }

    fn finish_export(
        &self,
        req: &ExportRequest,
        rel: &Path,
        out: &Path,
        (width, height): (u32, u32),
        path: ExportPath,
    ) -> CkResult<ExportResult> {
        let workdir = self.workdir();
        let info = ck_media::probe(out)?;
        let loudness = ck_media::measure_loudness(out)?;
        let silent = loudness.integrated <= ck_media::export::SILENT_LUFS;
        let loudness_pass = silent
            || ((loudness.integrated - TARGET.integrated).abs() <= LOUDNESS_TOLERANCE
                && loudness.true_peak <= TARGET.true_peak);
        // Spec 7.6: export prints a license and attribution report.
        let (license_report, license_ok) = match ck_assets::hub::Hub::locate(&workdir) {
            Ok(hub) => {
                let report = ck_assets::report::license_report(
                    &hub,
                    self.store.root(),
                    &ck_assets::report::LicenseReportRequest::default(),
                )?;
                let path = out.with_extension("licenses.json");
                let text = serde_json::to_vec_pretty(&report)
                    .map_err(|e| CkError::new(ErrorCode::Server, e.to_string()))?;
                ck_core::storage::atomic_write(&path, &text)?;
                let shown = path.strip_prefix(&workdir).map_or_else(
                    |_| path.to_string_lossy().into_owned(),
                    |p| p.to_string_lossy().replace('\\', "/"),
                );
                (Some(shown), Some(report.ok))
            }
            Err(_) => (None, None),
        };
        Ok(ExportResult {
            via: path,
            license_report,
            license_ok,
            path: rel.to_string_lossy().replace('\\', "/"),
            preset: req.preset,
            width,
            height,
            duration_sec: (info.duration * 1000.0).round() / 1000.0,
            loudness,
            loudness_pass,
            silent,
        })
    }
}

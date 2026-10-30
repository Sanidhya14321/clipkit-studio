//! `plan.edl` (docs/E2E_PIPELINE.md section 3): the rhythm grid, the agent's segments and their
//! expansion into layers. Each segment owns the layers whose ids start `edl-<id>-`, so changing
//! one segment re-expands only that segment.

use std::path::{Path, PathBuf};

use ck_assets::layout::{self, TextBlock};
use ck_assets::library;
use ck_assets::native_adapter::{self, NATIVE, NativeKind};
use ck_core::model::{Holder, Project, Source, TrackKind};
use ck_core::{CkError, CkResult, ErrorCode};
use ck_media::CachePaths;
use ck_pipeline::edl::{self, Grid, Placed, Role, Segment};
use ck_voice::{ContentMap, Word};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::edit::SourceRequest;
use super::import::ImportRequest;
use super::recipe::{PlaceMode, RecipeApplyRequest};
use super::{Session, is_zero};

const CAPTION_TRACK: &str = "captions-1";
/// Most words on screen in one caption.
const CAPTION_WORDS: usize = 3;
const CANDIDATES: usize = 3;
const SLAM: &str = "clipkit/kinetic-slam";
const BACKDROP: &str = "clipkit/backdrop";
/// Length of a transition, centred on the cut.
const TRANSITION_DUR: f64 = 0.4;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GridBuildRequest {
    /// Source whose speech sets the rhythm.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Audio source whose beats are measured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub music: Option<String>,
    /// Tempo of a generated grid; from the brief's pacing when omitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bpm: Option<f64>,
    /// Length of a generated grid in seconds; default 30.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dur: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GridResult {
    pub rev: u64,
    pub source: edl::GridSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tempo: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drop: Option<usize>,
    pub anchors: Vec<f64>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub omitted: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanSetRequest {
    pub segments: Vec<Segment>,
    /// Music asset laid under the whole plan and ducked under speech.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub music: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanExpandRequest {
    /// Only this segment; default all.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub segment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlanView {
    pub anchors: usize,
    pub source: edl::GridSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tempo: Option<f64>,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlanWriteResult {
    pub rev: u64,
    pub segments: usize,
    pub layers: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlanCandidates {
    /// `segment role: recipe | recipe | recipe`, best first.
    pub lines: Vec<String>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub omitted: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Stored {
    grid: Grid,
    #[serde(default)]
    segments: Vec<Segment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    music: Option<String>,
}

fn stored(project: &Project) -> CkResult<Stored> {
    serde_json::from_value(project.plan["edl"].clone()).map_err(|_| {
        CkError::new(ErrorCode::NotFound, "this project has no plan.edl yet")
            .with_hint("Run ck grid build first.")
    })
}

fn request(message: impl Into<String>) -> CkError {
    CkError::new(ErrorCode::Request, message)
}

fn server(e: impl std::fmt::Display) -> CkError {
    CkError::new(ErrorCode::Server, e.to_string())
}

fn role_query(role: Role) -> &'static str {
    match role {
        Role::Hook => "hook",
        Role::Point => "section step list",
        Role::Proof => "stat quote testimonial",
        Role::Turn => "reveal compare",
        Role::Cta => "cta outro",
    }
}

fn plan_op(project: &Project, plan: &Stored) -> CkResult<Value> {
    let value = serde_json::to_value(plan).map_err(server)?;
    let mut whole = project.plan.as_object().cloned().unwrap_or_default();
    whole.insert("edl".into(), value);
    Ok(json!([{ "op": "replace", "path": "/plan", "value": whole }]))
}

/// Short caption blocks from the words spoken inside `range`, in output time from `at`.
fn caption_blocks(words: &[Word], range: (f64, f64), at: f64, prefix: &str) -> Vec<TextBlock> {
    let kept: Vec<&Word> = words
        .iter()
        .filter(|w| (w.start + w.end) / 2.0 >= range.0 && (w.start + w.end) / 2.0 < range.1)
        .collect();
    let end = at + range.1 - range.0;
    let chunks: Vec<&[&Word]> = kept.chunks(CAPTION_WORDS).collect();
    chunks
        .iter()
        .enumerate()
        .map(|(n, c)| {
            let start = at + c[0].start - range.0;
            let stop = chunks.get(n + 1).map_or(end, |x| at + x[0].start - range.0);
            let text: Vec<&str> = c.iter().map(|w| w.text.as_str()).collect();
            TextBlock {
                id: format!("{prefix}cap-{}", n + 1),
                text: text.join(" "),
                role: layout::TextRole::Caption,
                align: layout::Align::Center,
                at: start,
                dur: stop - start,
            }
        })
        .collect()
}

fn unknown(asset: &str, what: &str) -> CkError {
    request(format!("{asset:?} is not a renderable {what} asset"))
        .with_hint("Run ck plan candidates for the assets that render.")
}

fn check_assets(s: &Segment) -> CkResult<()> {
    let pairs = [
        (&s.text_asset, NativeKind::Text, "text"),
        (&s.overlay, NativeKind::Overlay, "overlay"),
        (&s.transition, NativeKind::Transition, "transition"),
    ];
    for (asset, kind, what) in pairs {
        if let Some(a) = asset
            && native_adapter::clip_of(a, kind).is_none()
        {
            return Err(unknown(a, what));
        }
    }
    if let Some(a) = &s.sfx {
        media_file(a)?;
    }
    Ok(())
}

/// The folder that holds audio packs: `CK_ASSETS`, else `assets/` above the binary or here.
fn assets_root() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("CK_ASSETS").filter(|v| !v.is_empty()) {
        return Some(PathBuf::from(p));
    }
    let exe = std::env::current_exe().ok()?;
    let here = std::env::current_dir().ok();
    exe.ancestors()
        .map(Path::to_path_buf)
        .chain(
            here.iter()
                .flat_map(|h| h.ancestors().map(Path::to_path_buf)),
        )
        .map(|r| r.join("assets"))
        .find(|a| a.join("sfx-elements-audio").is_dir())
}

/// An audio asset `pack/name`: its media file and measured onset.
fn media_file(asset: &str) -> CkResult<(PathBuf, f64)> {
    let missing = || unknown(asset, "audio");
    let (pack, name) = asset.split_once('/').ok_or_else(missing)?;
    let safe = |p: &str| !p.is_empty() && p.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    if !safe(pack) || !safe(name) {
        return Err(missing());
    }
    let dir = assets_root().ok_or_else(missing)?.join(pack);
    let found = std::fs::read_dir(&dir)
        .map_err(|_| missing())?
        .flatten()
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n == name || n.starts_with(&format!("{name}-")))
        })
        .ok_or_else(missing)?;
    let file = std::fs::read_dir(found.join("media"))
        .map_err(|_| missing())?
        .flatten()
        .map(|e| e.path())
        .next()
        .ok_or_else(missing)?;
    let onset = std::fs::read_to_string(found.join("analysis.json"))
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|v| v["onsetSec"].as_f64())
        .unwrap_or(0.0);
    Ok((file, onset))
}

/// Short name of every pack entry, for candidates.
fn pack_names(pack: &str, words: &[&str]) -> Vec<String> {
    let Some(root) = assets_root() else {
        return Vec::new();
    };
    let mut names: Vec<String> = std::fs::read_dir(root.join(pack))
        .map(|d| {
            d.flatten()
                .filter(|e| e.path().join("media").is_dir())
                .filter_map(|e| e.file_name().to_str().map(str::to_owned))
                .filter_map(|n| n.rsplit_once('-').map(|(a, _)| a.to_owned()))
                .filter(|n| words.is_empty() || words.iter().any(|w| n.contains(w)))
                .map(|n| format!("{pack}/{n}"))
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names.truncate(CANDIDATES);
    names
}

fn natives(kind: NativeKind) -> String {
    let all: Vec<&str> = NATIVE.iter().filter(|n| n.3 == kind).map(|n| n.0).collect();
    all.join(" ")
}

fn provenance(asset: &str) -> Value {
    json!({ "sourceAsset": asset, "hash": format!("native:{asset}"), "mode": "native-adapter",
            "adaptations": ["text", "timing"], "license": "see license report" })
}

fn native_item(id: String, clip: &str, asset: &str, at: f64, dur: f64, props: Value) -> Value {
    json!({ "id": id, "name": clip, "purpose": "planned asset", "type": "clip",
            "clipId": clip, "at": at, "dur": dur, "props": props,
            "assetRef": asset, "provenance": provenance(asset) })
}

impl Session {
    /// Adds items to the first track of `kind`, creating it, and records `assets`.
    fn add_items(
        &self,
        (kind, track): (TrackKind, &str),
        items: Vec<Value>,
        assets: Vec<String>,
        label: &str,
    ) -> CkResult<()> {
        if items.is_empty() {
            return Ok(());
        }
        let mut doc = self.doc()?;
        let project = doc.project();
        let mut ops: Vec<Value> = match project.tracks.iter().position(|t| t.id == track) {
            Some(t) => items
                .into_iter()
                .map(|i| json!({ "op": "add", "path": format!("/tracks/{t}/items/-"), "value": i }))
                .collect(),
            None => {
                let name = serde_json::to_value(kind).map_err(server)?;
                let name = name.as_str().unwrap_or("overlay").to_owned();
                vec![json!({ "op": "add", "path": "/tracks/-", "value": {
                    "id": track, "kind": name, "items": items } })]
            }
        };
        let mut used = project.assets_used.clone();
        for a in assets {
            if !used.contains(&a) {
                ops.push(json!({ "op": "add", "path": "/assetsUsed/-", "value": a }));
                used.push(a);
            }
        }
        self.commit_agent(&mut doc, Value::Array(ops), label.to_owned())?;
        Ok(())
    }

    /// Imports an audio asset once and returns its source id, onset and length.
    fn audio_source(&self, asset: &str) -> CkResult<(String, f64, f64)> {
        let (file, onset) = media_file(asset)?;
        let done = self.import(&ImportRequest {
            path: file.to_string_lossy().into_owned(),
        })?;
        Ok((
            done.source.id,
            onset,
            done.source.duration_sec.unwrap_or(0.0),
        ))
    }

    /// Overlay, then text, then transition: the draw order inside the track.
    fn add_native(&self, s: &Segment, p: &Placed, prefix: &str) -> CkResult<usize> {
        let project = self.project()?;
        let mut items = Vec::new();
        let mut assets = Vec::new();
        let mut base = Vec::new();
        let slam = s.text_asset.as_deref() == Some(SLAM);
        if s.source.is_none() && s.recipe.is_none() && !slam {
            let props = json!({ "color": "#0b1020", "accent": "#22d3ee", "motif": true });
            let id = format!("{prefix}-backdrop");
            base.push(native_item(id, "backdrop", BACKDROP, p.at, p.dur, props));
            assets.push(BACKDROP.to_owned());
        }
        if let Some(a) = &s.overlay
            && let Some(clip) = native_adapter::clip_of(a, NativeKind::Overlay)
        {
            items.push(native_item(
                format!("{prefix}-overlay"),
                clip,
                a,
                p.at,
                p.dur,
                json!({}),
            ));
            assets.push(a.clone());
        }
        if let (Some(a), Some(text)) = (&s.text_asset, &s.text)
            && let Some(clip) = native_adapter::clip_of(a, NativeKind::Text)
        {
            let id = format!("{prefix}-text");
            let block = TextBlock {
                id: id.clone(),
                text: text.clone(),
                role: layout::TextRole::Title,
                align: layout::Align::Center,
                at: p.at,
                dur: p.dur,
            };
            let mut props = serde_json::Map::new();
            props.insert("text".into(), json!(text));
            props.insert("color".into(), json!("#ffffff"));
            if slam {
                // Colour flips every other bar; one motif, so it may repeat.
                let (ground, ink) = if s.from % 8 >= 4 {
                    ("#0a0a0a", "#ff5a00")
                } else {
                    ("#ff5a00", "#0a0a0a")
                };
                props.insert("ground".into(), json!(ground));
                props.insert("color".into(), json!(ink));
                props.insert("ink".into(), json!(ink));
                props.insert("motif".into(), json!(true));
            }
            let platform = super::layout::platform(&project);
            if let Some(placed) = layout::solve(platform, 1.0, &[block]).first() {
                layout::write_props(placed, &mut props);
            }
            let layer = native_item(id, clip, a, p.at, p.dur, Value::Object(props));
            if slam && s.source.is_none() {
                base.push(layer);
            } else {
                items.push(layer);
            }
            assets.push(a.clone());
        }
        if let Some(a) = &s.transition
            && let Some(clip) = native_adapter::clip_of(a, NativeKind::Transition)
            && p.at > 0.0
        {
            let at = (p.at - TRANSITION_DUR / 2.0).max(0.0);
            let id = format!("{prefix}-transition");
            items.push(native_item(id, clip, a, at, TRANSITION_DUR, json!({})));
            assets.push(a.clone());
        }
        let n = items.len() + base.len();
        self.add_items(
            (TrackKind::Video, "video-edl"),
            base,
            assets.clone(),
            "plan: base",
        )?;
        self.add_items(
            (TrackKind::Overlay, "overlay-edl"),
            items,
            assets,
            "plan: assets",
        )?;
        if let Some(a) = &s.sfx {
            let (src, onset, len) = self.audio_source(a)?;
            let item = json!({ "id": format!("{prefix}-sfx"), "name": "Sound", "purpose": "sound on the cut",
                "type": "audio", "src": src, "at": (p.at - onset).max(0.0),
                "ranges": [{ "start": 0.0, "end": len }], "volume": 0.4, "assetRef": a });
            self.add_items(
                (TrackKind::Audio, "audio-sfx"),
                vec![item],
                vec![a.clone()],
                "plan: sound",
            )?;
            return Ok(n + 1);
        }
        Ok(n)
    }

    /// The music bed under the whole plan, ducked under speech and voice.
    fn add_music(&self, asset: &str, total: f64) -> CkResult<()> {
        let (src, _, len) = self.audio_source(asset)?;
        let item = json!({ "id": "edl-music", "name": "Music", "purpose": "music bed",
            "type": "audio", "src": src, "at": 0.0,
            "ranges": [{ "start": 0.0, "end": len.min(total) }],
            "volume": 0.35, "duck": true, "assetRef": asset });
        self.add_items(
            (TrackKind::Audio, "audio-music"),
            vec![item],
            vec![asset.to_owned()],
            "plan: music",
        )
    }

    fn plan_source(&self, id: &str) -> CkResult<Source> {
        self.project()?
            .sources
            .iter()
            .find(|s| s.id == id)
            .cloned()
            .ok_or_else(|| CkError::new(ErrorCode::NotFound, format!("no source with id {id:?}")))
    }

    fn words(&self, src: &Source) -> CkResult<Vec<Word>> {
        Ok(self
            .transcript_words(&SourceRequest {
                source: src.id.clone(),
            })?
            .words)
    }

    /// `content map`: read-only.
    pub fn content_map(&self, req: &SourceRequest) -> CkResult<ContentMap> {
        let src = self.plan_source(&req.source)?;
        let t = self.transcript_words(req)?;
        Ok(ck_voice::content_map(&t, src.duration_sec.unwrap_or(0.0)))
    }

    fn music_beats(&self, id: &str) -> CkResult<ck_media::Beats> {
        let src = self.plan_source(id)?;
        let cache = CachePaths::new(&self.store.root().join("cache"), &src.hash)?;
        let path = if cache.audio.is_file() {
            cache.audio
        } else {
            self.resolve(&src.path)
        };
        ck_media::measure_beats(&path)
    }

    /// `grid build`: stores the grid, keeping segments that still fit it.
    pub fn grid_build(&self, req: &GridBuildRequest) -> CkResult<GridResult> {
        let project = self.project()?;
        let music = req
            .music
            .as_deref()
            .map(|m| self.music_beats(m))
            .transpose()?;
        let grid = if let Some(id) = &req.source {
            let t = self.transcript_words(&SourceRequest { source: id.clone() })?;
            let found = ck_voice::phrases(&t);
            if found.is_empty() {
                return Err(request(format!(
                    "source {id:?} has no speech to set a rhythm"
                )));
            }
            let spans: Vec<(f64, f64)> = found.iter().map(|p| (p.start, p.end)).collect();
            let mut g = edl::speech_grid(&spans, found.iter().filter_map(|p| p.emphasis).collect());
            g.music_beats = music.map(|m| m.beats).unwrap_or_default();
            g
        } else if let Some(m) = music {
            edl::music_grid(m.beats, m.tempo)
        } else {
            let pacing = project
                .brief
                .fields
                .get("pacing")
                .and_then(|f| f.value.as_str());
            let bpm = req.bpm.unwrap_or_else(|| edl::pace_bpm(pacing));
            let dur = req.dur.unwrap_or(30.0);
            if !(bpm > 0.0 && dur > 0.0 && dur <= 3600.0) {
                return Err(request("bpm and dur must be positive, dur at most 3600"));
            }
            edl::generated_grid(dur, bpm)
        };
        let old = stored(&project).map(|s| s.segments).unwrap_or_default();
        let segments = if edl::check(&grid, &old).is_ok() {
            old
        } else {
            Vec::new()
        };
        let plan = Stored {
            grid: grid.clone(),
            segments,
            music: None,
        };
        let mut doc = self.doc()?;
        let ops = plan_op(doc.project(), &plan)?;
        let rev = self.commit_evidence(&mut doc, ops, "plan: rhythm grid".into())?;
        Ok(GridResult {
            rev: rev.rev,
            source: grid.source,
            tempo: grid.tempo,
            drop: grid.drop,
            anchors: grid.anchors,
            omitted: 0,
        })
    }

    pub fn plan_get(&self) -> CkResult<PlanView> {
        let plan = stored(&self.project()?)?;
        Ok(PlanView {
            anchors: plan.grid.anchors.len(),
            source: plan.grid.source,
            tempo: plan.grid.tempo,
            segments: plan.segments,
        })
    }

    pub fn plan_set(&self, req: &PlanSetRequest) -> CkResult<PlanWriteResult> {
        let mut doc = self.doc()?;
        let mut plan = stored(doc.project())?;
        edl::check(&plan.grid, &req.segments)?;
        for s in &req.segments {
            if let Some(r) = &s.recipe {
                library::recipe_with_text(r)?;
            }
        }
        for s in &req.segments {
            check_assets(s)?;
        }
        if let Some(m) = &req.music {
            media_file(m)?;
        }
        plan.segments = req.segments.clone();
        plan.music = req.music.clone();
        let ops = plan_op(doc.project(), &plan)?;
        let rev = self.commit_agent(&mut doc, ops, "plan: segments".into())?;
        Ok(PlanWriteResult {
            rev: rev.rev,
            segments: req.segments.len(),
            layers: 0,
        })
    }

    /// Top recipes by role, those already chosen elsewhere twice left out.
    pub fn plan_candidates(&self) -> CkResult<PlanCandidates> {
        let plan = stored(&self.project()?)?;
        let mut lines = Vec::new();
        for s in &plan.segments {
            let used = |id: &str| {
                plan.segments
                    .iter()
                    .filter(|o| o.recipe.as_deref() == Some(id))
                    .count()
            };
            let hits = library::recipe_search(&library::RecipeSearchRequest {
                query: role_query(s.role).into(),
                limit: Some(12),
            })?;
            let names: Vec<String> = hits
                .hits
                .iter()
                .filter(|h| used(&h.id) < 2)
                .take(CANDIDATES)
                .map(|h| format!("{} ({})", h.id, h.slots.join(",")))
                .collect();
            lines.push(format!("{} {:?}: {}", s.id, s.role, names.join(" | ")));
        }
        lines.push(format!("text: {}", natives(NativeKind::Text)));
        lines.push(format!("overlay: {}", natives(NativeKind::Overlay)));
        lines.push(format!("transition: {}", natives(NativeKind::Transition)));
        lines.push(format!(
            "sfx: {}",
            pack_names("sfx-elements-audio", &["whoosh", "pop", "boom"]).join(" ")
        ));
        lines.push(format!(
            "music: {}",
            pack_names("duru-original-music", &[]).join(" ")
        ));
        Ok(PlanCandidates { lines, omitted: 0 })
    }

    /// Removes the layers `prefix` owns from every track.
    fn drop_layers(&self, prefix: &str) -> CkResult<()> {
        let mut doc = self.doc()?;
        let mut ops = Vec::new();
        for (t, track) in doc.project().tracks.iter().enumerate() {
            if track.items.iter().any(|i| i.id.starts_with(prefix)) {
                let items: Vec<_> = track
                    .items
                    .iter()
                    .filter(|i| !i.id.starts_with(prefix))
                    .collect();
                ops.push(json!({ "op": "replace", "path": format!("/tracks/{t}/items"), "value": items }));
            }
        }
        if !ops.is_empty() {
            self.commit_agent(&mut doc, Value::Array(ops), format!("plan: clear {prefix}"))?;
        }
        Ok(())
    }

    /// The caption layers of one segment, placed in the caption band.
    fn add_captions(&self, blocks: Vec<TextBlock>) -> CkResult<usize> {
        let mut doc = self.doc()?;
        let project = doc.project();
        let platform = super::layout::platform(project);
        let solved = layout::solve(platform, 1.0, &blocks);
        let mut items = Vec::new();
        for (b, p) in blocks.iter().zip(&solved) {
            let mut props = serde_json::Map::new();
            props.insert("text".into(), json!(b.text));
            props.insert("size".into(), json!("small"));
            props.insert("role".into(), json!("caption"));
            props.insert("color".into(), json!("#ffffff"));
            layout::write_props(p, &mut props);
            items.push(json!({
                "id": b.id, "name": "Caption", "purpose": "words of the kept speech",
                "type": "clip", "clipId": "kinetic-text", "at": b.at, "dur": b.dur, "props": props,
            }));
        }
        let n = items.len();
        let ops = match project
            .tracks
            .iter()
            .position(|t| t.kind == TrackKind::Caption)
        {
            Some(t) => items
                .into_iter()
                .map(|i| json!({ "op": "add", "path": format!("/tracks/{t}/items/-"), "value": i }))
                .collect(),
            None => vec![json!({ "op": "add", "path": "/tracks/-", "value": {
                "id": CAPTION_TRACK, "kind": "caption", "items": items,
            }})],
        };
        self.commit_agent(&mut doc, Value::Array(ops), "plan: captions".into())?;
        Ok(n)
    }

    fn expand_one(&self, s: &Segment, p: &Placed) -> CkResult<usize> {
        let prefix = format!("edl-{}", s.id);
        let mut count = 0;
        if let Some(recipe) = &s.recipe {
            let done = self.recipe_apply_inner(&RecipeApplyRequest {
                recipe: recipe.clone(),
                at: p.at,
                duration: Some(p.dur),
                place: PlaceMode::Overlay,
                slots: s.slots.clone(),
                prefix: Some(prefix.clone()),
                look: None,
            })?;
            count += done.layers.len() + done.omitted;
        }
        count += self.add_native(s, p, &prefix)?;
        if let (true, Some(id), Some(range)) = (s.captions, &s.source, p.range) {
            let words = self.words(&self.plan_source(id)?)?;
            let blocks = caption_blocks(&words, range, p.at, &format!("{prefix}-"));
            if !blocks.is_empty() {
                count += self.add_captions(blocks)?;
            }
        }
        Ok(count)
    }

    /// `plan expand`: all segments (footage, then layers), or one segment's layers only.
    pub fn plan_expand(&self, req: &PlanExpandRequest) -> CkResult<PlanWriteResult> {
        let project = self.project()?;
        self.gate_agent_write(&project)?;
        let plan = stored(&project)?;
        edl::check(&plan.grid, &plan.segments)?;
        let placed = edl::place(&plan.grid, &plan.segments, project.settings.fps);
        let pairs: Vec<(&Segment, &Placed)> = plan.segments.iter().zip(&placed).collect();
        let mut layers = 0;
        if let Some(id) = &req.segment {
            let (s, p) = pairs
                .iter()
                .find(|(s, _)| &s.id == id)
                .ok_or_else(|| CkError::new(ErrorCode::NotFound, format!("no segment {id:?}")))?;
            self.drop_layers(&format!("edl-{id}-"))?;
            layers += self.expand_one(s, p)?;
        } else {
            self.drop_layers("edl-")?;
            let source = plan.segments.iter().find_map(|s| s.source.clone());
            if let Some(source) = source {
                let kept: Vec<(f64, f64)> = placed.iter().filter_map(|p| p.range).collect();
                let mut doc = self.doc()?;
                let (ops, track, _) = super::edit::footage_ops(doc.project(), &source, &kept);
                self.gate_agent_write(doc.project())?;
                let label = "plan: kept speech".to_owned();
                let ops = Value::Array(ops);
                self.commit_as(&mut doc, Holder::Agent, ops, label, &[track])?;
            }
            for (s, p) in &pairs {
                layers += self.expand_one(s, p)?;
            }
            if let (Some(m), Some(last)) = (&plan.music, placed.last()) {
                self.add_music(m, last.at + last.dur)?;
                layers += 1;
            }
        }
        Ok(PlanWriteResult {
            rev: self.project()?.rev,
            segments: if req.segment.is_some() {
                1
            } else {
                pairs.len()
            },
            layers,
        })
    }
}

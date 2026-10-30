//! `plan.edl` (docs/E2E_PIPELINE.md section 3): the rhythm grid, the agent's segments and their
//! expansion into layers. Each segment owns the layers whose ids start `edl-<id>-`, so changing
//! one segment re-expands only that segment.

use ck_assets::layout::{self, TextBlock};
use ck_assets::library;
use ck_core::model::{Holder, Project, Source, TrackKind};
use ck_core::{CkError, CkResult, ErrorCode};
use ck_media::CachePaths;
use ck_pipeline::edl::{self, Grid, Placed, Role, Segment};
use ck_voice::{ContentMap, Word};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::edit::SourceRequest;
use super::recipe::{PlaceMode, RecipeApplyRequest};
use super::{Session, is_zero};

const CAPTION_TRACK: &str = "captions-1";
/// Most words on screen in one caption.
const CAPTION_WORDS: usize = 3;
const CANDIDATES: usize = 3;

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

impl Session {
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
        plan.segments = req.segments.clone();
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

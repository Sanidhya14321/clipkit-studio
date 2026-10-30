//! `recipe apply` (spec 7.1, 7.5): expand a recipe with slot values into named layers and write
//! them as one agent revision. Each layer goes on the first track of its kind (a new track when
//! there is none), carries its provenance, and the recipe joins `assetsUsed`.

use std::collections::BTreeMap;

use ck_assets::layout::{self, Platform, TextBlock};
use ck_assets::library;
use ck_assets::recipe::{ExpandInput, ExpandedLayer};
use ck_core::model::{Author, Holder, ItemBody, Project, TrackKind};
use ck_core::patch::PatchRequest;
use ck_core::{CkError, CkResult, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::{Session, is_zero};
use crate::event::Event;

/// Look used when neither the request nor the brief names one.
pub const DEFAULT_LOOK: &str = "clean-product";
const MAX_PREFIX: usize = 40;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecipeApplyRequest {
    /// Recipe id (from `recipe search`).
    pub recipe: String,
    /// Scene start in seconds.
    #[serde(default)]
    pub at: f64,
    /// Scene duration in seconds; the recipe default when omitted.
    #[serde(
        default,
        rename = "dur",
        alias = "duration",
        skip_serializing_if = "Option::is_none"
    )]
    pub duration: Option<f64>,
    /// How the scene meets what is already at `at` (default overlay).
    #[serde(default, skip_serializing_if = "PlaceMode::is_overlay")]
    pub place: PlaceMode,
    /// Slot name to value (text, number, list of text, or a source id).
    #[serde(default)]
    pub slots: BTreeMap<String, Value>,
    /// Layer id prefix; layer ids are `<prefix>-<layer name>`. Defaults to the recipe id, with
    /// `-2`, `-3` ... added when that is taken.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    /// Look id whose tokens the layers use; defaults to the project look (`look apply`), then
    /// the brief's look, then `clean-product`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub look: Option<String>,
}

/// Where a scene goes relative to existing layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum PlaceMode {
    /// Layers on top; nothing moves.
    #[default]
    Overlay,
    /// Every later layer moves by the scene's full length.
    Insert,
    /// Later layers move only as far as needed to clear the scene.
    Ripple,
}

impl PlaceMode {
    fn is_overlay(&self) -> bool {
        *self == Self::Overlay
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecipeApplyResult {
    pub rev: u64,
    pub look: String,
    /// Ids of the layers added, in recipe order.
    pub layers: Vec<String>,
    /// Layer ids left out of `layers` to keep the reply under its size cap.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub omitted: usize,
}

fn request_err(message: String) -> CkError {
    CkError::new(ErrorCode::Request, message)
}

fn server_err(e: impl std::fmt::Display) -> CkError {
    CkError::new(ErrorCode::Server, e.to_string())
}

/// The look to use: the request's, else the project's (stored, brief, default).
fn look_id(req: &RecipeApplyRequest, project: &Project) -> String {
    req.look
        .clone()
        .unwrap_or_else(|| super::look::project_look_id(project))
}

fn item_ids(project: &Project) -> Vec<&str> {
    project
        .tracks
        .iter()
        .flat_map(|t| t.items.iter().map(|i| i.id.as_str()))
        .collect()
}

/// The requested prefix (refused when any layer id it makes is taken, so a retry never
/// duplicates), or the recipe id with the first free number.
fn prefix(req: &RecipeApplyRequest, project: &Project, names: &[&str]) -> CkResult<String> {
    let ids = item_ids(project);
    let free = |p: &str| {
        !names
            .iter()
            .any(|n| ids.contains(&format!("{p}-{n}").as_str()))
    };
    if let Some(p) = &req.prefix {
        let ok = !p.is_empty()
            && p.len() <= MAX_PREFIX
            && p.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
        if !ok {
            return Err(request_err(format!(
                "prefix {p:?} must be 1 to {MAX_PREFIX} letters, digits or -"
            )));
        }
        if !free(p) {
            return Err(request_err(format!(
                "prefix {p:?} makes layer ids that already exist; pick another or omit it"
            )));
        }
        return Ok(p.clone());
    }
    let base = req.recipe.as_str();
    if free(base) {
        return Ok(base.to_owned());
    }
    (2..)
        .map(|n| format!("{base}-{n}"))
        .find(|p| free(p))
        .ok_or_else(|| server_err("no free prefix"))
}

fn kind_name(kind: TrackKind) -> &'static str {
    match kind {
        TrackKind::Video => "video",
        TrackKind::Overlay => "overlay",
        TrackKind::Audio => "audio",
        TrackKind::Caption => "caption",
    }
}

/// Solves the scene's text layers together and writes each placement (`box`, `fontSize`,
/// `lines`, `textCut`) into its props.
fn place_text(platform: Platform, type_scale: f64, layers: &mut [ExpandedLayer]) {
    let blocks: Vec<TextBlock> = layers
        .iter()
        .filter_map(|l| match &l.item.body {
            ItemBody::Clip { at, dur, props, .. } => {
                layout::text_block(&l.item.id, *at, *dur, props)
            }
            _ => None,
        })
        .collect();
    for p in layout::solve(platform, type_scale, &blocks) {
        let props = layers.iter_mut().find_map(|l| match &mut l.item.body {
            ItemBody::Clip { props, .. } if l.item.id == p.id => props.as_object_mut(),
            _ => None,
        });
        if let Some(props) = props {
            layout::write_props(&p, props);
        }
    }
}

/// True when any track holds footage (EDIT or HYBRID base).
pub fn has_footage(project: &Project) -> bool {
    project
        .tracks
        .iter()
        .flat_map(|t| &t.items)
        .any(|i| matches!(i.body, ItemBody::Footage { .. }))
}

/// RFC 6902 operations that add the layers (and any missing track) and record the recipe.
/// How far `place` moves the layers starting at or after `at`.
fn place_shift(project: &Project, place: PlaceMode, at: f64, dur: f64) -> f64 {
    let next = || {
        project
            .tracks
            .iter()
            .flat_map(|t| &t.items)
            .filter_map(|i| serde_json::to_value(i).ok()?["at"].as_f64())
            .filter(|a| *a >= at)
            .fold(f64::INFINITY, f64::min)
    };
    match place {
        PlaceMode::Overlay => 0.0,
        PlaceMode::Insert => dur,
        PlaceMode::Ripple => (at + dur - next()).max(0.0),
    }
}

fn build_ops(
    project: &Project,
    layers: Vec<ExpandedLayer>,
    asset: &str,
    (from, shift): (f64, f64),
) -> CkResult<Vec<Value>> {
    let shift_of = |a: f64| if a >= from { shift } else { 0.0 };
    let mut placed: BTreeMap<usize, usize> = BTreeMap::new();
    let mut ops = Vec::new();
    if shift > 0.0 {
        for (t, track) in project.tracks.iter().enumerate() {
            for (i, item) in track.items.iter().enumerate() {
                let at = serde_json::to_value(item).map_err(server_err)?["at"].as_f64();
                if let Some(a) = at.filter(|a| *a >= from) {
                    ops.push(json!({ "op": "replace", "path": format!("/tracks/{t}/items/{i}/at"), "value": a + shift }));
                }
            }
        }
    }
    // New tracks are appended in first-use order; `new_tracks` holds each one's kind and index.
    let mut new_tracks: Vec<(TrackKind, usize)> = Vec::new();
    let mut track_ids: Vec<String> = project.tracks.iter().map(|t| t.id.clone()).collect();
    // HYBRID (spec 4): generated layers go over the footage, never onto its track.
    let over_footage = has_footage(project);
    for mut layer in layers {
        if over_footage && layer.track == TrackKind::Video {
            layer.track = TrackKind::Overlay;
        }
        let item = serde_json::to_value(&layer.item).map_err(server_err)?;
        let index = project
            .tracks
            .iter()
            .position(|t| t.kind == layer.track)
            .or_else(|| {
                new_tracks
                    .iter()
                    .find(|(k, _)| *k == layer.track)
                    .map(|(_, i)| *i)
            });
        match index {
            // Items stay sorted by time: after everything that starts at or before this one.
            Some(i) => {
                let at = item["at"].as_f64().unwrap_or(f64::MAX);
                let before = project.tracks.get(i).map_or(0, |t| {
                    t.items
                        .iter()
                        .filter(|o| {
                            serde_json::to_value(o).is_ok_and(|v| {
                                v["at"].as_f64().is_none_or(|a| a + shift_of(a) <= at)
                            })
                        })
                        .count()
                });
                let added = placed.entry(i).or_insert(0);
                ops.push(json!({ "op": "add", "path": format!("/tracks/{i}/items/{}", before + *added), "value": item }));
                *added += 1;
            }
            None => {
                let name = kind_name(layer.track);
                let id = (1..)
                    .map(|n| format!("{name}-{n}"))
                    .find(|id| !track_ids.contains(id))
                    .ok_or_else(|| server_err("no free track id"))?;
                track_ids.push(id.clone());
                new_tracks.push((layer.track, track_ids.len() - 1));
                ops.push(json!({
                    "op": "add",
                    "path": "/tracks/-",
                    "value": { "id": id, "kind": name, "items": [item] }
                }));
            }
        }
    }
    if !project.assets_used.iter().any(|a| a == asset) {
        ops.push(json!({ "op": "add", "path": "/assetsUsed/-", "value": asset }));
    }
    Ok(ops)
}

impl Session {
    /// `recipe apply`: expand and write the layers as one revision by the agent. The agent
    /// briefly takes the locks the write touches unless it holds them, so a human mid-edit
    /// refuses the write (E_LOCK_HELD) instead of being overwritten.
    pub fn recipe_apply(&self, req: &RecipeApplyRequest) -> CkResult<RecipeApplyResult> {
        self.with_retry("recipe apply", req, || self.recipe_apply_inner(req))
    }

    pub(super) fn recipe_apply_inner(
        &self,
        req: &RecipeApplyRequest,
    ) -> CkResult<RecipeApplyResult> {
        let (recipe, text) = library::recipe_with_text(&req.recipe)?;
        let now = self.now_ms();
        let mut doc = self.doc()?;
        self.gate_agent_write(doc.project())?;
        let project = doc.project();
        let look_id = look_id(req, project);
        let look = library::look_get(&library::IdRequest {
            id: look_id.clone(),
        })?;
        let names: Vec<&str> = recipe.layers.iter().map(|l| l.name.as_str()).collect();
        let prefix = prefix(req, project, &names)?;
        let sources: Vec<String> = project.sources.iter().map(|s| s.id.clone()).collect();
        let slots: serde_json::Map<String, Value> = req.slots.clone().into_iter().collect();
        let expansion = recipe.expand(
            &look,
            text,
            &ExpandInput {
                prefix: &prefix,
                at: req.at,
                duration: req.duration,
                slots: &slots,
                sources: &sources,
            },
        )?;
        let mut layers = expansion.layers;
        place_text(
            super::layout::platform(project),
            look.type_scale,
            &mut layers,
        );
        let layer_ids: Vec<String> = layers.iter().map(|l| l.item.id.clone()).collect();
        let dur = req.duration.unwrap_or(recipe.duration.default);
        let shift = place_shift(project, req.place, req.at, dur);
        let ops = build_ops(
            project,
            layers,
            &expansion.provenance.source_asset,
            (req.at, shift),
        )?;
        let patch = PatchRequest {
            base_rev: project.rev,
            layer_revs: Default::default(),
            ops: serde_json::from_value(Value::Array(ops)).map_err(server_err)?,
            author: Author::Agent,
            label: format!("recipe: {}", recipe.id),
        };
        // Dry run to learn the lock ids this write touches.
        let touched = ck_core::patch::apply(project, &patch)?.touched;
        let missing: Vec<String> = touched
            .into_iter()
            .filter(|id| {
                !project
                    .locks
                    .iter()
                    .any(|l| &l.layer_id == id && l.holder == Holder::Agent && l.is_live(now))
            })
            .collect();
        if !missing.is_empty() {
            doc.acquire(Holder::Agent, &missing, now)?;
        }
        let committed = doc.patch(&patch, now).cloned();
        if !missing.is_empty() {
            doc.release(Holder::Agent, Some(&missing));
        }
        let rev = committed?;
        self.note_write(&mut doc);
        self.persist(&mut doc)?;
        self.publish(Event::Revision {
            rev: rev.rev,
            author: rev.author,
            label: rev.label,
            touched: rev.touched,
        });
        self.publish_snapshot(&doc);
        Ok(RecipeApplyResult {
            rev: rev.rev,
            look: look_id,
            layers: layer_ids,
            omitted: 0,
        })
    }
}

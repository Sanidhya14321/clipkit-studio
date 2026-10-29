//! `recipe apply` (spec 7.1, 7.5): expand a recipe with slot values into named layers and write
//! them as one agent revision. Each layer goes on the first track of its kind (a new track when
//! there is none), carries its provenance, and the recipe joins `assetsUsed`.

use std::collections::BTreeMap;

use ck_assets::library;
use ck_assets::recipe::ExpandInput;
use ck_core::model::{Author, Holder, Project, TrackKind};
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    /// Slot name to value (text, number, list of text, or a source id).
    #[serde(default)]
    pub slots: BTreeMap<String, Value>,
    /// Layer id prefix; layer ids are `<prefix>-<layer name>`. Defaults to the recipe id, with
    /// `-2`, `-3` ... added when that is taken.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    /// Look id whose tokens the layers use; defaults to the brief's look, then `clean-product`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub look: Option<String>,
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

/// The look to use: the request's, else the brief's `look` field, else the default.
fn look_id(req: &RecipeApplyRequest, project: &Project) -> String {
    req.look
        .clone()
        .or_else(|| {
            project
                .brief
                .fields
                .get("look")
                .and_then(|f| f.value.as_str())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| DEFAULT_LOOK.into())
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

/// RFC 6902 operations that add the layers (and any missing track) and record the recipe.
fn build_ops(
    project: &Project,
    layers: Vec<ck_assets::recipe::ExpandedLayer>,
    asset: &str,
) -> CkResult<Vec<Value>> {
    let mut ops = Vec::new();
    // New tracks are appended in first-use order; `new_tracks` holds each one's kind and index.
    let mut new_tracks: Vec<(TrackKind, usize)> = Vec::new();
    let mut track_ids: Vec<String> = project.tracks.iter().map(|t| t.id.clone()).collect();
    for layer in layers {
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
            Some(i) => ops.push(
                json!({ "op": "add", "path": format!("/tracks/{i}/items/-"), "value": item }),
            ),
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
        let (recipe, text) = library::recipe_with_text(&req.recipe)?;
        let now = self.now_ms();
        let mut doc = self.doc()?;
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
        let layer_ids: Vec<String> = expansion.layers.iter().map(|l| l.item.id.clone()).collect();
        let ops = build_ops(
            project,
            expansion.layers,
            &expansion.provenance.source_asset,
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

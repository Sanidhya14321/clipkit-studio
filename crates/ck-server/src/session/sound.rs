//! `sound plan` (spec 8.5): deterministic sound design placement. Whooshes land slightly before
//! cuts, hits on text landings, risers lead into hooks, room tone sits under cuts and music
//! under voice is ducked. The plan is read-only: the agent places the cues it keeps in the
//! audio pass. No sfx cards are in the catalog yet, so cues carry timing and no asset.

use ck_core::CkResult;
use ck_core::model::{Item, ItemBody, Project, TrackKind};
use ck_core::timeline::span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Session, is_zero};

/// A whoosh starts this long before the cut so its peak meets the cut.
pub const WHOOSH_LEAD: f64 = 0.15;
pub const WHOOSH_DUR: f64 = 0.4;
/// A text layer lands this long after it starts entering (before the look's duration scale).
pub const HIT_DELAY: f64 = 0.2;
pub const RISER_DUR: f64 = 1.0;
/// Room tone covers this long on each side of a cut.
pub const ROOM_TONE_PAD: f64 = 0.25;

const EPS: f64 = 1e-6;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum CueKind {
    Whoosh,
    Hit,
    Riser,
    RoomTone,
    Duck,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SoundCue {
    pub kind: CueKind,
    /// Start in seconds.
    pub at: f64,
    pub dur: f64,
    /// The layer the cue follows (the cut's incoming layer, the text, the hook or the music).
    pub layer: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SoundPlan {
    pub rev: u64,
    /// Cues in time order.
    pub cues: Vec<SoundCue>,
    /// Cues left out to keep the reply under its size cap.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub omitted: usize,
}

fn round(t: f64) -> f64 {
    (t * 1000.0).round() / 1000.0
}

fn cue(kind: CueKind, at: f64, dur: f64, layer: &str) -> SoundCue {
    let start = at.max(0.0);
    SoundCue {
        kind,
        at: round(start),
        dur: round((at + dur - start).max(0.0)),
        layer: layer.to_owned(),
    }
}

fn is_text(item: &Item) -> bool {
    matches!(&item.body, ItemBody::Clip { props, .. } if props.get("text").and_then(Value::as_str).is_some())
}

/// A hook layer: its name, purpose or recipe says `hook`.
fn is_hook(item: &Item) -> bool {
    let source = item.provenance.as_ref().map(|p| p.source_asset.as_str());
    [
        Some(item.name.as_str()),
        Some(item.purpose.as_str()),
        source,
    ]
    .into_iter()
    .flatten()
    .any(|s| s.to_ascii_lowercase().contains("hook"))
}

/// The cue plan for a project. `duration_scale` is the look's motion scale for text landings.
pub fn sound_plan(project: &Project, duration_scale: f64) -> SoundPlan {
    let mut cues = Vec::new();
    for track in &project.tracks {
        let mut spans: Vec<(f64, f64, &Item)> = track
            .items
            .iter()
            .filter_map(|i| span(project, i).map(|s| (s.start, s.end, i)))
            .collect();
        spans.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.2.id.cmp(&b.2.id)));
        if track.kind == TrackKind::Video {
            // A cut is where an item starts after the timeline start.
            for (start, _, item) in spans.iter().filter(|s| s.0 > EPS) {
                cues.push(cue(
                    CueKind::Whoosh,
                    start - WHOOSH_LEAD,
                    WHOOSH_DUR,
                    &item.id,
                ));
                cues.push(cue(
                    CueKind::RoomTone,
                    start - ROOM_TONE_PAD,
                    2.0 * ROOM_TONE_PAD,
                    &item.id,
                ));
            }
        }
        for (start, end, item) in &spans {
            if is_text(item) {
                let land = start + HIT_DELAY * duration_scale;
                if land < *end {
                    cues.push(cue(CueKind::Hit, land, 0.0, &item.id));
                }
            }
            if is_hook(item) && *start > EPS {
                cues.push(cue(CueKind::Riser, start - RISER_DUR, RISER_DUR, &item.id));
            }
        }
    }
    // Music not yet ducked, under each stretch of voice it overlaps.
    let voices: Vec<(f64, f64)> = project
        .items()
        .filter(|i| matches!(i.body, ItemBody::Voice { .. }))
        .filter_map(|i| span(project, i).map(|s| (s.start, s.end)))
        .collect();
    for item in project.items() {
        if !matches!(item.body, ItemBody::Audio { duck: false, .. }) {
            continue;
        }
        let Some(music) = span(project, item) else {
            continue;
        };
        for (vs, ve) in &voices {
            let (s, e) = (vs.max(music.start), ve.min(music.end));
            if e > s + EPS {
                cues.push(cue(CueKind::Duck, s, e - s, &item.id));
            }
        }
    }
    cues.sort_by(|a, b| {
        a.at.total_cmp(&b.at)
            .then(a.kind.cmp(&b.kind))
            .then(a.layer.cmp(&b.layer))
    });
    cues.dedup();
    SoundPlan {
        rev: project.rev,
        cues,
        omitted: 0,
    }
}

impl Session {
    /// `sound plan`: read-only.
    pub fn sound_plan(&self) -> CkResult<SoundPlan> {
        let project = self.project()?;
        let look = ck_assets::library::look_get(&ck_assets::library::IdRequest {
            id: super::look::project_look_id(&project),
        })?;
        Ok(sound_plan(&project, look.motion.duration_scale))
    }
}

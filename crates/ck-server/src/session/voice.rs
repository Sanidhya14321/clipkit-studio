//! Voice slots (spec 8.3): the script split into slots with target windows, each AI, recorded
//! or cloned and switchable at any time. Each slot has a Voice item on the voice track at its
//! window start; the item's length comes from the slot's active take.

use ck_core::model::{
    Item, ItemBody, Project, TimeRange, Track, TrackKind, VoiceMode, VoiceSlot, VoiceSlotStatus,
};
use ck_core::{CkError, CkResult, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::Session;

/// Id of the track that holds the voice items.
pub const VOICE_TRACK: &str = "voice-1";
/// Words per second used to estimate a window when the script gives none.
pub const WORDS_PER_SEC: f64 = 2.5;
/// Pause between estimated windows, in seconds.
pub const LINE_GAP: f64 = 0.3;
/// Most slots in one script.
pub const MAX_LINES: usize = 200;
/// Longest line, in bytes.
pub const MAX_LINE: usize = 1000;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScriptLine {
    pub text: String,
    /// Target window; estimated from the word count when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window: Option<TimeRange>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VoiceScriptRequest {
    /// One slot per line, in order. Replaces the current slots; a slot whose text and mode are
    /// unchanged keeps its takes.
    pub lines: Vec<ScriptLine>,
    /// Mode for new slots (default ai).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<VoiceMode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SlotSummary {
    pub id: String,
    pub mode: VoiceMode,
    pub status: VoiceSlotStatus,
    pub window: Option<TimeRange>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VoiceScriptResult {
    pub rev: u64,
    pub slots: Vec<SlotSummary>,
    #[serde(default, skip_serializing_if = "super::is_zero")]
    pub omitted: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VoiceModeRequest {
    pub slot: String,
    pub mode: VoiceMode,
    /// Who switches it: the agent (default) or the human (the studio badge).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<ck_core::model::Author>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VoiceModeResult {
    pub rev: u64,
    pub slot: SlotSummary,
}

fn round3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

pub fn summary(s: &VoiceSlot) -> SlotSummary {
    SlotSummary {
        id: s.id.clone(),
        mode: s.mode,
        status: s.status,
        window: s.target_window,
    }
}

/// The slots for `req`, keeping takes of unchanged lines from `old`.
pub fn slots_for(req: &VoiceScriptRequest, old: &[VoiceSlot]) -> CkResult<Vec<VoiceSlot>> {
    if req.lines.is_empty() || req.lines.len() > MAX_LINES {
        return Err(CkError::new(
            ErrorCode::Request,
            format!("a script has 1 to {MAX_LINES} lines"),
        ));
    }
    let mode = req.mode.unwrap_or(VoiceMode::Ai);
    let mut at = 0.0;
    let mut out = Vec::new();
    for (n, line) in req.lines.iter().enumerate() {
        let text = line.text.trim();
        if text.is_empty() || text.len() > MAX_LINE {
            return Err(CkError::new(
                ErrorCode::Request,
                format!("line {n} must have 1 to {MAX_LINE} bytes of text"),
            ));
        }
        let window = match line.window {
            Some(w) if w.end > w.start && w.start >= 0.0 => w,
            Some(_) => {
                return Err(CkError::new(
                    ErrorCode::Request,
                    format!("line {n} has an empty or negative window"),
                ));
            }
            None => {
                let words = text.split_whitespace().count() as f64;
                TimeRange {
                    start: round3(at),
                    end: round3(at + words / WORDS_PER_SEC),
                }
            }
        };
        at = window.end + LINE_GAP;
        let id = format!("v{}", n + 1);
        let kept = old
            .iter()
            .find(|s| s.id == id && s.text == text)
            .filter(|s| line.window.is_none() || s.target_window == Some(window));
        out.push(match kept {
            Some(s) => VoiceSlot {
                target_window: Some(window),
                ..s.clone()
            },
            None => VoiceSlot {
                id,
                text: text.to_owned(),
                mode,
                voice_ref: None,
                takes: Vec::new(),
                active_take: None,
                target_window: Some(window),
                status: VoiceSlotStatus::Empty,
            },
        });
    }
    Ok(out)
}

/// The voice track holding one Voice item per slot at its window start.
pub fn voice_track(slots: &[VoiceSlot]) -> Track {
    Track {
        id: VOICE_TRACK.into(),
        kind: TrackKind::Audio,
        items: slots
            .iter()
            .map(|s| Item {
                id: format!("voice-{}", s.id),
                name: format!("Voice {}", s.id),
                purpose: "narration".into(),
                pass: None,
                asset_ref: None,
                brief_deps: Vec::new(),
                provenance: None,
                rev: 0,
                body: ItemBody::Voice {
                    slot_id: s.id.clone(),
                    at: s.target_window.map_or(0.0, |w| w.start),
                },
            })
            .collect(),
    }
}

fn slot_index(project: &Project, id: &str) -> CkResult<usize> {
    project
        .voice_slots
        .iter()
        .position(|s| s.id == id)
        .ok_or_else(|| {
            CkError::new(ErrorCode::NotFound, format!("no voice slot {id:?}"))
                .with_hint("Run ck voice script first; slot ids are v1, v2, ...")
        })
}

impl Session {
    /// `voice script`: one revision replacing the slots and the voice track.
    pub fn voice_script(&self, req: &VoiceScriptRequest) -> CkResult<VoiceScriptResult> {
        let mut doc = self.doc()?;
        let project = doc.project();
        let slots = slots_for(req, &project.voice_slots)?;
        let track = voice_track(&slots);
        // Stored items carry the rev that last touched them; compare without it.
        let same_track = project.tracks.iter().any(|t| {
            t.id == VOICE_TRACK
                && t.items.len() == track.items.len()
                && t.items
                    .iter()
                    .zip(&track.items)
                    .all(|(a, b)| a.id == b.id && a.body == b.body)
        });
        if slots == project.voice_slots && same_track {
            return Ok(VoiceScriptResult {
                rev: project.rev,
                slots: slots.iter().map(summary).collect(),
                omitted: 0,
            });
        }
        let mut ops = vec![json!({ "op": "replace", "path": "/voiceSlots", "value": slots })];
        match project.tracks.iter().position(|t| t.id == VOICE_TRACK) {
            Some(i) => ops.push(json!({ "op": "replace", "path": format!("/tracks/{i}"),
                                        "value": track })),
            None => ops.push(json!({ "op": "add", "path": "/tracks/-", "value": track })),
        }
        // Voice items of dropped slots would dangle: the whole track is rebuilt, so none do.
        let label = format!("voice script: {} slots", slots.len());
        let rev = self.commit_agent(&mut doc, serde_json::Value::Array(ops), label)?;
        Ok(VoiceScriptResult {
            rev: rev.rev,
            slots: slots.iter().map(summary).collect(),
            omitted: 0,
        })
    }

    /// `voice mode`: switch a slot between AI, recorded and cloned; takes are kept.
    pub fn voice_mode(&self, req: &VoiceModeRequest) -> CkResult<VoiceModeResult> {
        let holder = match req.author {
            Some(ck_core::model::Author::Human | ck_core::model::Author::Voice) => {
                ck_core::model::Holder::Human
            }
            _ => ck_core::model::Holder::Agent,
        };
        let mut doc = self.doc()?;
        let project = doc.project();
        let n = slot_index(project, &req.slot)?;
        let mut slot = project.voice_slots[n].clone();
        if slot.mode == req.mode {
            return Ok(VoiceModeResult {
                rev: project.rev,
                slot: summary(&slot),
            });
        }
        slot.mode = req.mode;
        let ops = json!([{ "op": "replace", "path": format!("/voiceSlots/{n}/mode"),
                           "value": req.mode }]);
        let label = format!("voice mode: {} {:?}", req.slot, req.mode).to_lowercase();
        let rev = self.commit_as(&mut doc, holder, ops, label, &[])?;
        Ok(VoiceModeResult {
            rev: rev.rev,
            slot: summary(&slot),
        })
    }
}

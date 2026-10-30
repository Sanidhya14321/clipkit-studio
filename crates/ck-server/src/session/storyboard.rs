//! Storyboard checkpoint (spec 4 PLAN, 6.2 `storyboard get, confirm`): before building the
//! passes, the agent shows one still per scene; the user confirms in the studio and the
//! agent's `wait` sees the checkpoint. Requested by `storyboard` in the brief's checkpoints;
//! confirmed is recorded as `storyboard:confirmed`, by the user only.

use ck_core::model::Holder;
use ck_core::{CkError, CkResult, ErrorCode, Project};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::Session;

pub const CHECKPOINT: &str = "storyboard";
pub const CONFIRMED: &str = "storyboard:confirmed";

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoryboardRequest {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StoryboardResult {
    pub rev: u64,
    /// Scene start times, seconds.
    pub scenes: Vec<f64>,
    /// The sheet, relative to the project folder.
    pub sheet: String,
    pub requested: bool,
    pub confirmed: bool,
}

/// Scene starts: the starts of the base (video) track's items, where each recipe puts its
/// backdrop and footage sits; without a base track, every clip start. At least 0.5 s apart.
pub fn scene_starts(project: &Project) -> Vec<f64> {
    use ck_core::model::{ItemBody, TrackKind};
    let base: Vec<&ck_core::model::Item> = project
        .tracks
        .iter()
        .filter(|t| t.kind == TrackKind::Video)
        .flat_map(|t| &t.items)
        .collect();
    let items: Vec<&ck_core::model::Item> = if base.is_empty() {
        project
            .items()
            .filter(|i| matches!(i.body, ItemBody::Clip { .. } | ItemBody::Footage { .. }))
            .collect()
    } else {
        base
    };
    let mut starts: Vec<f64> = items
        .into_iter()
        .filter_map(|i| ck_core::timeline::span(project, i))
        .map(|s| s.start)
        .collect();
    starts.sort_by(f64::total_cmp);
    starts.dedup_by(|a, b| (*a - *b).abs() < 0.5);
    starts
}

impl Session {
    /// `storyboard get`: one still per scene, 1 s in (or mid-scene if shorter), as a sheet.
    pub fn storyboard_get(&self, _req: &StoryboardRequest) -> CkResult<StoryboardResult> {
        let project = self.project()?;
        let starts = scene_starts(&project);
        if starts.is_empty() {
            return Err(CkError::new(ErrorCode::Request, "no scenes yet")
                .with_hint("Apply the scene recipes first, then show the storyboard."));
        }
        let end = ck_core::timeline::duration(&project);
        let times: Vec<f64> = starts
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let next = starts.get(i + 1).copied().unwrap_or(end);
                (s + 1.0).min((s + next) / 2.0)
            })
            .collect();
        let rel = self.stills_sheet(&project, &times, "storyboard")?;
        let b = &project.brief;
        Ok(StoryboardResult {
            rev: project.rev,
            scenes: starts,
            sheet: rel,
            requested: b.checkpoints.iter().any(|c| c == CHECKPOINT),
            confirmed: b.checkpoints.iter().any(|c| c == CONFIRMED),
        })
    }

    /// The user confirms the storyboard (studio only).
    pub fn storyboard_confirm(&self) -> CkResult<StoryboardResult> {
        let mut doc = self.doc()?;
        let p = doc.project();
        let rev = if p.brief.checkpoints.iter().any(|c| c == CONFIRMED) {
            p.rev
        } else {
            let ops = json!([{ "op": "add", "path": "/brief/checkpoints/-", "value": CONFIRMED }]);
            self.commit_as(
                &mut doc,
                Holder::Human,
                ops,
                "storyboard confirmed".into(),
                &[],
            )?
            .rev
        };
        let p = doc.project();
        Ok(StoryboardResult {
            rev,
            scenes: scene_starts(p),
            sheet: format!(".clipkit/storyboard/sheet-{}.png", p.rev),
            requested: p.brief.checkpoints.iter().any(|c| c == CHECKPOINT),
            confirmed: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scenes_start_at_distinct_clip_and_footage_starts() {
        let mut p = Project::new("p", "P", ck_core::model::Mode::Generate);
        p.tracks = serde_json::from_value(json!([
            { "id": "v", "kind": "video", "items": [
                { "id": "a", "type": "clip", "at": 0.0, "dur": 4.0 },
                { "id": "b", "type": "clip", "at": 4.0, "dur": 5.0 } ] },
            { "id": "o", "kind": "overlay", "items": [
                { "id": "t", "type": "clip", "at": 0.1, "dur": 2.0 },
                { "id": "u", "type": "clip", "at": 9.0, "dur": 6.0 } ] }
        ]))
        .unwrap();
        // The base track decides: the overlay's 9 s title is inside the second scene's span
        // only when there is no base; here the video track has two scenes.
        assert_eq!(scene_starts(&p), [0.0, 4.0]);
        p.tracks.remove(0);
        assert_eq!(scene_starts(&p), [0.1, 9.0]);
    }
}

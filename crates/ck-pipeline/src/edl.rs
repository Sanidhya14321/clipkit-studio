//! The Edit Decision Plan (docs/E2E_PIPELINE.md section 3), stored as `plan.edl`. The agent
//! decides per segment, in anchor indices; this code alone turns anchors into exact times.

use std::collections::BTreeMap;

use ck_core::{CkError, CkResult, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Beats per minute of a generated grid, by brief pacing. Fast unless the brief says otherwise.
pub fn pace_bpm(pacing: Option<&str>) -> f64 {
    match pacing.map(str::to_ascii_lowercase) {
        Some(p) if ["slow", "calm", "cinematic"].iter().any(|w| p.contains(w)) => 80.0,
        Some(p)
            if ["medium", "moderate", "steady"]
                .iter()
                .any(|w| p.contains(w)) =>
        {
            100.0
        }
        _ => 120.0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum GridSource {
    Speech,
    Music,
    Voice,
    Generated,
}

/// The rhythm grid. Segments point into `anchors`; for speech they are source times.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Grid {
    pub source: GridSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tempo: Option<f64>,
    /// Beats, or phrase boundaries for speech, seconds.
    pub anchors: Vec<f64>,
    /// Anchor indices of downbeats.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub downbeats: Vec<usize>,
    /// How far each anchor may move inside its pause (speech only), seconds.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub slack: Vec<f64>,
    /// Emphasis word times, seconds.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub emphasis: Vec<f64>,
    /// Anchor index of the turning point.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drop: Option<usize>,
    /// Music beats in output time, when music is laid under speech.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub music_beats: Vec<f64>,
}

fn every_fourth(n: usize) -> Vec<usize> {
    (0..n).step_by(4).collect()
}

/// A code-made grid: `bpm` from 0 to `duration`, turning point at about two thirds.
pub fn generated_grid(duration: f64, bpm: f64) -> Grid {
    let step = 60.0 / bpm;
    let count = (duration / step).ceil() as usize;
    let anchors: Vec<f64> = (0..=count)
        .map(|i| ((i as f64 * step).min(duration) * 1000.0).round() / 1000.0)
        .collect();
    Grid {
        source: GridSource::Generated,
        tempo: Some(bpm),
        downbeats: every_fourth(anchors.len()),
        drop: Some(anchors.len() * 2 / 3 / 4 * 4),
        anchors,
        slack: Vec::new(),
        emphasis: Vec::new(),
        music_beats: Vec::new(),
    }
}

/// A grid on measured beats.
pub fn music_grid(beats: Vec<f64>, tempo: f64) -> Grid {
    Grid {
        source: GridSource::Music,
        tempo: Some(tempo),
        downbeats: every_fourth(beats.len()),
        drop: Some(beats.len() * 2 / 3 / 4 * 4),
        anchors: beats,
        slack: Vec::new(),
        emphasis: Vec::new(),
        music_beats: Vec::new(),
    }
}

/// A grid on speech: `phrases` are `(start, end)` in source time; boundaries sit mid-pause.
pub fn speech_grid(phrases: &[(f64, f64)], emphasis: Vec<f64>) -> Grid {
    let mut anchors = Vec::with_capacity(phrases.len() + 1);
    let mut slack = Vec::with_capacity(phrases.len() + 1);
    if let Some((s, _)) = phrases.first() {
        anchors.push(*s);
        slack.push(0.0);
    }
    for pair in phrases.windows(2) {
        let (end, next) = (pair[0].1, pair[1].0);
        anchors.push((end + next) / 2.0);
        slack.push((next - end) / 2.0);
    }
    if let Some((_, e)) = phrases.last() {
        anchors.push(*e);
        slack.push(0.0);
    }
    Grid {
        source: GridSource::Speech,
        tempo: None,
        anchors,
        downbeats: Vec::new(),
        slack,
        emphasis,
        drop: None,
        music_beats: Vec::new(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    Hook,
    Point,
    Proof,
    Turn,
    Cta,
}

/// One decision of the agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Segment {
    pub id: String,
    /// First anchor index.
    pub from: usize,
    /// Anchor index where the segment ends; after `from`.
    pub to: usize,
    pub role: Role,
    /// Recipe drawn over the segment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipe: Option<String>,
    /// Recipe slot values.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub slots: BTreeMap<String, Value>,
    /// Source id whose speech this keeps; anchors are then that source's times.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Transition asset played across this segment's start (at most two types per plan).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transition: Option<String>,
    /// Words on screen, drawn with `textAsset`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Text animation asset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_asset: Option<String>,
    /// Full-frame overlay asset for the segment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overlay: Option<String>,
    /// Sound effect asset; its onset lands on the segment start.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sfx: Option<String>,
    /// Draw captions from the kept words.
    #[serde(default)]
    pub captions: bool,
}

/// Where a segment lands: output time and, for kept speech, the source range.
#[derive(Debug, Clone, PartialEq)]
pub struct Placed {
    pub at: f64,
    pub dur: f64,
    pub range: Option<(f64, f64)>,
}

fn request(message: String) -> CkError {
    CkError::new(ErrorCode::Request, message)
        .with_hint("Fix the segment and run ck plan set again.")
}

/// Refuses a plan the expander could not make exact.
pub fn check(grid: &Grid, segments: &[Segment]) -> CkResult<()> {
    let mut last = 0;
    let mut ids = std::collections::BTreeSet::new();
    let mut uses: BTreeMap<&str, usize> = BTreeMap::new();
    let mut transitions = std::collections::BTreeSet::new();
    let speech = segments.iter().filter(|s| s.source.is_some()).count();
    if speech != 0 && speech != segments.len() {
        return Err(request(
            "mix of kept speech and generated segments; use one kind per plan".into(),
        ));
    }
    for s in segments {
        let ok_id = !s.id.is_empty()
            && s.id.len() <= 24
            && s.id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
        if !ok_id || !ids.insert(s.id.as_str()) {
            return Err(request(format!(
                "segment id {:?} must be unique, 1 to 24 letters, digits or -",
                s.id
            )));
        }
        if s.to <= s.from || s.to >= grid.anchors.len() {
            return Err(request(format!(
                "segment {:?} range {}..{} is outside the grid's {} anchors",
                s.id,
                s.from,
                s.to,
                grid.anchors.len()
            )));
        }
        if s.from < last {
            return Err(request(format!(
                "segment {:?} overlaps the one before",
                s.id
            )));
        }
        last = s.to;
        if let Some(r) = &s.recipe {
            *uses.entry(r).or_default() += 1;
            if uses[r.as_str()] > 2 {
                return Err(request(format!("recipe {r:?} used more than twice")));
            }
        }
        transitions.extend(s.transition.as_deref());
    }
    if transitions.len() > 2 {
        return Err(request("more than two transition types".into()));
    }
    Ok(())
}

/// Nearest music beat within `slack` of `t`, else `t`.
fn on_beat(t: f64, slack: f64, beats: &[f64]) -> f64 {
    beats
        .iter()
        .copied()
        .filter(|b| (b - t).abs() <= slack)
        .min_by(|a, b| (a - t).abs().total_cmp(&(b - t).abs()))
        .unwrap_or(t)
}

fn frame(t: f64, fps: u32) -> f64 {
    let fps = f64::from(fps);
    (t * fps).round() / fps
}

/// Exact placement of every segment, on whole frames. Kept speech is laid end to end, each cut
/// moved inside its pause onto a music beat when there is one.
pub fn place(grid: &Grid, segments: &[Segment], fps: u32) -> Vec<Placed> {
    let mut cursor = 0.0;
    segments
        .iter()
        .map(|s| {
            let (a, b) = (grid.anchors[s.from], grid.anchors[s.to]);
            if s.source.is_none() {
                let at = frame(a, fps);
                return Placed {
                    at,
                    dur: frame(b, fps) - at,
                    range: None,
                };
            }
            let slack = grid.slack.get(s.to).copied().unwrap_or(0.0);
            let end = on_beat(cursor + b - a, slack, &grid.music_beats);
            let at = frame(cursor, fps);
            let dur = frame(end, fps) - at;
            cursor = at + dur;
            Placed {
                at,
                dur,
                range: Some((a, a + dur)),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seg(id: &str, from: usize, to: usize) -> Segment {
        Segment {
            id: id.into(),
            from,
            to,
            role: Role::Point,
            recipe: None,
            slots: BTreeMap::new(),
            source: None,
            transition: None,
            text: None,
            text_asset: None,
            overlay: None,
            sfx: None,
            captions: false,
        }
    }

    #[test]
    fn fast_is_the_default_pace() {
        assert_eq!(pace_bpm(None), 120.0);
        assert_eq!(pace_bpm(Some("fast")), 120.0);
        assert_eq!(pace_bpm(Some("calm cinematic")), 80.0);
    }

    #[test]
    fn generated_grid_covers_the_length_exactly() {
        let g = generated_grid(10.0, 120.0);
        assert_eq!(g.anchors.len(), 21);
        assert_eq!(g.anchors[20], 10.0);
        assert_eq!(g.downbeats[1], 4);
        assert_eq!(g.drop.unwrap() % 4, 0);
    }

    #[test]
    fn check_refuses_bad_plans() {
        let g = generated_grid(10.0, 120.0);
        assert!(check(&g, &[seg("a", 0, 4), seg("b", 4, 8)]).is_ok());
        assert!(check(&g, &[seg("a", 0, 4), seg("a", 4, 8)]).is_err());
        assert!(check(&g, &[seg("a", 0, 4), seg("b", 3, 8)]).is_err());
        assert!(check(&g, &[seg("a", 0, 99)]).is_err());
        let mut hog = vec![seg("a", 0, 1), seg("b", 1, 2), seg("c", 2, 3)];
        for s in &mut hog {
            s.recipe = Some("x".into());
        }
        assert!(check(&g, &hog).is_err());
    }

    #[test]
    fn placement_lands_on_whole_frames() {
        let g = generated_grid(10.0, 113.0);
        for p in place(&g, &[seg("a", 0, 3), seg("b", 3, 7)], 30) {
            for t in [p.at, p.at + p.dur] {
                assert!(((t * 30.0).round() - t * 30.0).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn speech_cuts_sit_in_pauses_and_move_to_a_beat() {
        let mut g = speech_grid(&[(1.0, 2.0), (2.4, 3.0)], vec![]);
        assert_eq!(g.anchors[1], 2.2);
        let mut s = seg("a", 0, 1);
        s.source = Some("v".into());
        assert_eq!(place(&g, &[s.clone()], 30)[0].dur, 1.2);
        g.music_beats = vec![1.1];
        let p = &place(&g, &[s], 30)[0];
        assert!((p.dur - 1.1).abs() < 1e-3);
        assert_eq!(p.range.unwrap().0, 1.0);
    }
}

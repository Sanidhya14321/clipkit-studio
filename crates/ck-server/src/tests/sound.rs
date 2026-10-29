//! `sound plan` (spec 8.5): cue placement per kind, clamping at the timeline start.

use ck_core::Project;
use ck_core::model::Mode;
use serde_json::{Value, json};

use crate::session::sound::{CueKind, HIT_DELAY, SoundCue, sound_plan};

fn clip(id: &str, name: &str, at: f64, dur: f64, props: Value) -> Value {
    json!({ "id": id, "name": name, "type": "clip", "clipId": "backdrop", "at": at, "dur": dur,
            "props": props })
}

fn project(tracks: Value) -> Project {
    let mut p = Project::new("p1", "Demo", Mode::Generate);
    p.tracks = serde_json::from_value(tracks).unwrap();
    p
}

fn kinds(cues: &[SoundCue], kind: CueKind) -> Vec<(f64, f64, &str)> {
    cues.iter()
        .filter(|c| c.kind == kind)
        .map(|c| (c.at, c.dur, c.layer.as_str()))
        .collect()
}

#[test]
fn cuts_get_whooshes_and_room_tone_and_text_gets_hits() {
    let p = project(json!([
        { "id": "video-1", "kind": "video", "items": [
            clip("a", "Scene backdrop", 0.0, 2.0, json!({})),
            clip("b", "Scene backdrop", 2.0, 2.0, json!({})) ] },
        { "id": "overlay-1", "kind": "overlay", "items": [
            clip("t", "Scene headline", 0.5, 1.0, json!({ "text": "Hi" })) ] }
    ]));
    let plan = sound_plan(&p, 1.0);
    // The first item is not a cut; the second is.
    assert_eq!(kinds(&plan.cues, CueKind::Whoosh), [(1.85, 0.4, "b")]);
    assert_eq!(kinds(&plan.cues, CueKind::RoomTone), [(1.75, 0.5, "b")]);
    assert_eq!(kinds(&plan.cues, CueKind::Hit), [(0.7, 0.0, "t")]);
    assert!(plan.cues.windows(2).all(|w| w[0].at <= w[1].at));

    // The look's duration scale stretches the landing.
    let slow = sound_plan(&p, 2.0);
    assert_eq!(
        kinds(&slow.cues, CueKind::Hit),
        [(0.5 + 2.0 * HIT_DELAY, 0.0, "t")]
    );
}

#[test]
fn risers_lead_into_hooks_and_clamp_at_zero() {
    let p = project(json!([
        { "id": "overlay-1", "kind": "overlay", "items": [
            clip("h0", "Opening hook", 0.0, 1.0, json!({})),
            clip("h1", "Hook title", 0.4, 1.0, json!({})),
            clip("h2", "Second hook", 3.0, 1.0, json!({})),
            clip("x", "Outro", 5.0, 1.0, json!({})) ] }
    ]));
    let plan = sound_plan(&p, 1.0);
    // A hook at 0 has no lead-in; one at 0.4 gets a riser clamped to start at 0.
    assert_eq!(
        kinds(&plan.cues, CueKind::Riser),
        [(0.0, 0.4, "h1"), (2.0, 1.0, "h2")]
    );
}

#[test]
fn music_under_voice_gets_duck_cues_unless_already_ducked() {
    let mut p = project(json!([
        { "id": "audio-1", "kind": "audio", "items": [
            { "id": "music", "type": "audio", "src": "s1", "at": 0.0,
              "ranges": [{ "start": 0.0, "end": 6.0 }] },
            { "id": "ducked", "type": "audio", "src": "s1", "at": 0.0, "duck": true,
              "ranges": [{ "start": 0.0, "end": 6.0 }] } ] },
        { "id": "audio-2", "kind": "audio", "items": [
            { "id": "vo", "type": "voice", "slotId": "slot1", "at": 1.0 } ] }
    ]));
    p.voice_slots = serde_json::from_value(json!([
        { "id": "slot1", "text": "Hello", "mode": "ai", "activeTake": "k1",
          "takes": [{ "id": "k1", "path": "voice/k1.wav", "durationSec": 2.5 }] }
    ]))
    .unwrap();
    let plan = sound_plan(&p, 1.0);
    assert_eq!(kinds(&plan.cues, CueKind::Duck), [(1.0, 2.5, "music")]);
}

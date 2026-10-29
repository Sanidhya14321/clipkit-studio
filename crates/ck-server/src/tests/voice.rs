//! Voice slots (spec 8.3) through the session.

use ck_core::ErrorCode;
use ck_core::model::{Author, ItemBody, TimeRange, VoiceMode, VoiceSlotStatus, VoiceTake};
use serde_json::json;

use super::fixture::Fixture;
use crate::session::voice::{
    ScriptLine, VOICE_TRACK, VoiceModeRequest, VoiceScriptRequest, slots_for,
};

fn line(text: &str) -> ScriptLine {
    ScriptLine {
        text: text.into(),
        window: None,
    }
}

#[test]
fn a_script_becomes_slots_with_windows_and_voice_items() {
    let f = Fixture::new();
    let r = f
        .session
        .voice_script(&VoiceScriptRequest {
            lines: vec![
                line("Meet ClipKit, the editor you talk to."),
                ScriptLine {
                    text: "Cut by deleting words.".into(),
                    window: Some(TimeRange {
                        start: 5.0,
                        end: 7.0,
                    }),
                },
            ],
            mode: None,
        })
        .unwrap();
    assert_eq!(r.slots.len(), 2);
    // 7 words at 2.5 words/s.
    assert_eq!(
        r.slots[0].window,
        Some(TimeRange {
            start: 0.0,
            end: 2.8
        })
    );
    assert_eq!(r.slots[1].window.unwrap().start, 5.0);
    assert_eq!(r.slots[0].status, VoiceSlotStatus::Empty);
    assert_eq!(r.slots[0].mode, VoiceMode::Ai);

    let p = f.session.project().unwrap();
    let track = p.tracks.iter().find(|t| t.id == VOICE_TRACK).unwrap();
    let ats: Vec<f64> = track
        .items
        .iter()
        .map(|i| match i.body {
            ItemBody::Voice { at, .. } => at,
            _ => -1.0,
        })
        .collect();
    assert_eq!(ats, [0.0, 5.0]);

    // A second script keeps the voice track (no second one) and replaces the slots.
    f.session
        .voice_script(&VoiceScriptRequest {
            lines: vec![line("Only one line now.")],
            mode: Some(VoiceMode::Recorded),
        })
        .unwrap();
    let p = f.session.project().unwrap();
    assert_eq!(p.tracks.iter().filter(|t| t.id == VOICE_TRACK).count(), 1);
    assert_eq!(p.voice_slots.len(), 1);
    assert_eq!(p.voice_slots[0].mode, VoiceMode::Recorded);
}

#[test]
fn unchanged_lines_keep_their_takes() {
    let req = VoiceScriptRequest {
        lines: vec![line("Hello there."), line("Second line.")],
        mode: None,
    };
    let mut old = slots_for(&req, &[]).unwrap();
    old[0].takes.push(VoiceTake {
        id: "t1".into(),
        path: "a.wav".into(),
        duration_sec: 1.0,
    });
    old[0].active_take = Some("t1".into());
    let changed = VoiceScriptRequest {
        lines: vec![line("Hello there."), line("A new second line.")],
        mode: None,
    };
    let new = slots_for(&changed, &old).unwrap();
    assert_eq!(new[0].takes.len(), 1);
    assert!(new[1].takes.is_empty());
}

#[test]
fn modes_switch_and_bad_scripts_are_refused() {
    let f = Fixture::new();
    f.session
        .voice_script(&VoiceScriptRequest {
            lines: vec![line("Hi.")],
            mode: None,
        })
        .unwrap();
    let r = f
        .session
        .voice_mode(&VoiceModeRequest {
            slot: "v1".into(),
            mode: VoiceMode::Cloned,
            author: Some(Author::Human),
        })
        .unwrap();
    assert_eq!(r.slot.mode, VoiceMode::Cloned);
    let missing = f
        .session
        .voice_mode(&VoiceModeRequest {
            slot: "v9".into(),
            mode: VoiceMode::Ai,
            author: None,
        })
        .unwrap_err();
    assert_eq!(missing.code, ErrorCode::NotFound);

    for bad in [
        json!({ "lines": [] }),
        json!({ "lines": [{ "text": "  " }] }),
        json!({ "lines": [{ "text": "x", "window": { "start": 2.0, "end": 1.0 } }] }),
    ] {
        let req: VoiceScriptRequest = serde_json::from_value(bad).unwrap();
        assert_eq!(
            f.session.voice_script(&req).unwrap_err().code,
            ErrorCode::Request
        );
    }
}

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
        recorded: false,
        consented: false,
        transcript_path: None,
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

/// Writes a 440 Hz tone with 0.3 s of silence around it, 0.4 s per word.
struct ToneSpeaker;

impl ck_voice::Speaker for ToneSpeaker {
    fn speak(
        &self,
        text: &str,
        _voice: Option<&str>,
        out: &std::path::Path,
    ) -> ck_core::CkResult<()> {
        let secs = 0.4 * text.split_whitespace().count() as f64;
        let filter = format!("sine=f=440:d={secs},adelay=300,apad=pad_dur=0.3");
        let ok = std::process::Command::new("ffmpeg")
            .args([
                "-v", "error", "-y", "-f", "lavfi", "-i", &filter, "-f", "wav",
            ])
            .arg(out)
            .status()
            .unwrap()
            .success();
        assert!(ok);
        Ok(())
    }

    fn clone_voice(&self, audio: &std::path::Path) -> ck_core::CkResult<String> {
        assert!(audio.is_file(), "clones from the take file");
        Ok("fake-voice-1".into())
    }
}

fn speaking_session() -> (tempfile::TempDir, crate::session::Session) {
    let dir = tempfile::tempdir().unwrap();
    ck_core::storage::Store::new(dir.path())
        .init(&super::fixture::project())
        .unwrap();
    let s = crate::session::Session::open(
        dir.path(),
        std::sync::Arc::new(super::fixture::FakeClock::default()),
    )
    .unwrap()
    .with_speaker(std::sync::Arc::new(ToneSpeaker));
    (dir, s)
}

#[test]
fn speak_makes_a_trimmed_fitted_cached_take() {
    use crate::session::voice::SlotRequest;

    let (dir, s) = speaking_session();
    s.voice_script(&VoiceScriptRequest {
        // 5 words: 2.0 s at 2.5 words/s; the tone is 2.0 s after trimming.
        lines: vec![line("One two three four five.")],
        mode: None,
    })
    .unwrap();
    let slot = SlotRequest { slot: "v1".into() };
    let first = s.voice_speak(&slot).unwrap();
    assert!(!first.placeholder && !first.cached);
    let info = first.info.unwrap();
    assert!((info.raw_sec - 2.0).abs() < 0.1, "{info:?}");
    assert_eq!(info.fit, ck_media::Fit::Fits);
    let p = s.project().unwrap();
    let v1 = &p.voice_slots[0];
    assert_eq!(
        (v1.status, v1.active_take.as_deref()),
        (VoiceSlotStatus::Ready, Some("t1"))
    );
    assert!(dir.path().join("cache/takes/v1-t1.wav").is_file());
    // The voice item now spans the take.
    let item = p.item("voice-v1").unwrap();
    let span = ck_core::timeline::span(&p, item).unwrap();
    assert!((span.end - span.start - v1.takes[0].duration_sec).abs() < 1e-9);

    let second = s.voice_speak(&slot).unwrap();
    assert!(second.cached);
    assert_eq!(second.take.as_deref(), Some("t2"));
}

#[test]
fn without_a_speaker_the_slot_falls_back_to_the_placeholder() {
    use crate::session::voice::SlotRequest;

    let f = Fixture::new();
    f.session
        .voice_script(&VoiceScriptRequest {
            lines: vec![line("Hello.")],
            mode: None,
        })
        .unwrap();
    let r = f
        .session
        .voice_speak(&SlotRequest { slot: "v1".into() })
        .unwrap();
    assert!(r.placeholder);
    assert_eq!(r.slot.status, VoiceSlotStatus::Failed);
    assert!(r.reason.is_some());

    // Recorded slots are made in the studio; cloned ones need a cloned voice.
    for (mode, code) in [
        (VoiceMode::Recorded, ErrorCode::Request),
        (VoiceMode::Cloned, ErrorCode::Request),
    ] {
        f.session
            .voice_mode(&VoiceModeRequest {
                slot: "v1".into(),
                mode,
                author: None,
            })
            .unwrap();
        let err = f
            .session
            .voice_speak(&SlotRequest { slot: "v1".into() })
            .unwrap_err();
        assert_eq!(err.code, code);
    }
}

#[test]
fn a_studio_recording_becomes_a_recorded_take_and_takes_can_be_switched() {
    use crate::session::voice::ActiveTakeRequest;

    let fx = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/generated/tones.wav");
    if !fx.is_file() {
        eprintln!("skipped: run `just fixtures` first");
        return;
    }
    let f = Fixture::new();
    f.session
        .voice_script(&VoiceScriptRequest {
            lines: vec![line("Record me.")],
            mode: Some(VoiceMode::Recorded),
        })
        .unwrap();
    let bytes = std::fs::read(&fx).unwrap();
    let first = f.session.voice_record("v1", &bytes, "wav").unwrap();
    let second = f.session.voice_record("v1", &bytes, "wav").unwrap();
    assert_eq!(
        (first.take.as_deref(), second.take.as_deref()),
        (Some("t1"), Some("t2"))
    );
    let p = f.session.project().unwrap();
    let v1 = &p.voice_slots[0];
    assert!(v1.takes.iter().all(|t| t.recorded));
    assert_eq!(v1.active_take.as_deref(), Some("t2"));
    // 10 s of tone in a 0.8 s window: kept and reported too long.
    assert_eq!(first.info.unwrap().fit, ck_media::Fit::TooLong);

    f.session
        .voice_active_take(&ActiveTakeRequest {
            slot: "v1".into(),
            take: "t1".into(),
        })
        .unwrap();
    assert_eq!(
        f.session.project().unwrap().voice_slots[0]
            .active_take
            .as_deref(),
        Some("t1")
    );
    let missing = f
        .session
        .voice_active_take(&ActiveTakeRequest {
            slot: "v1".into(),
            take: "t9".into(),
        })
        .unwrap_err();
    assert_eq!(missing.code, ErrorCode::NotFound);

    let empty = f.session.voice_record("v1", &[], "wav").unwrap_err();
    assert_eq!(empty.code, ErrorCode::Request);
    let junk = f
        .session
        .voice_record("v1", b"not audio", "wav")
        .unwrap_err();
    assert_eq!(junk.code, ErrorCode::Media);
    let unknown = f.session.voice_record("v9", &bytes, "wav").unwrap_err();
    assert_eq!(unknown.code, ErrorCode::NotFound);
}

#[test]
fn cloning_needs_a_consented_studio_recording() {
    use crate::session::voice::{ConsentRequest, SlotRequest, VoiceCloneRequest};
    use ck_core::patch::PatchRequest;

    let fx = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/generated/tones.wav");
    if !fx.is_file() {
        eprintln!("skipped: run `just fixtures` first");
        return;
    }
    let (_dir, s) = speaking_session();
    s.voice_script(&VoiceScriptRequest {
        lines: vec![line("My own voice."), line("Generated line here.")],
        mode: None,
    })
    .unwrap();
    s.voice_record("v1", &std::fs::read(&fx).unwrap(), "wav")
        .unwrap();
    let clone = |slot: &str| {
        s.voice_clone(&VoiceCloneRequest {
            slot: slot.into(),
            take: None,
        })
    };

    // No consent yet.
    assert_eq!(clone("v1").unwrap_err().code, ErrorCode::Request);
    // The agent cannot give consent itself, even with a raw patch and the lock.
    s.lock(&crate::session::LockRequest {
        holder: ck_core::model::Holder::Agent,
        layer_ids: vec![ck_core::patch::PROJECT_LAYER.into()],
    })
    .unwrap();
    let rev = s.project().unwrap().rev;
    let sneaky = s
        .patch(&PatchRequest {
            base_rev: rev,
            layer_revs: Default::default(),
            ops: serde_json::from_value(json!([{ "op": "add",
                "path": "/voiceSlots/0/takes/0/consented", "value": true }]))
            .unwrap(),
            author: Author::Agent,
            label: "consent".into(),
        })
        .unwrap_err();
    assert_eq!(sneaky.code, ErrorCode::PatchInvalid);
    s.unlock(&crate::session::UnlockRequest {
        holder: ck_core::model::Holder::Agent,
        layer_ids: None,
    })
    .unwrap();

    // The user consents in the studio; cloning works and the slot turns cloned.
    s.voice_consent(&ConsentRequest {
        slot: "v1".into(),
        take: "t1".into(),
        consent: true,
    })
    .unwrap();
    let cloned = clone("v1").unwrap();
    assert_eq!(cloned.voice_ref, "fake-voice-1");
    assert_eq!(cloned.slot.mode, VoiceMode::Cloned);
    // A cloned slot now speaks through its voice.
    let spoken = s.voice_speak(&SlotRequest { slot: "v1".into() }).unwrap();
    assert!(!spoken.placeholder);

    // A generated take is never cloned.
    s.voice_speak(&SlotRequest { slot: "v2".into() }).unwrap();
    assert_eq!(clone("v2").unwrap_err().code, ErrorCode::Request);
    // Consent applies to studio recordings only.
    let err = s
        .voice_consent(&ConsentRequest {
            slot: "v2".into(),
            take: "t1".into(),
            consent: true,
        })
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::Request);
}

/// Three words over the first 1.2 s of any take; counts its calls.
#[derive(Default)]
struct WordsTranscriber(std::sync::atomic::AtomicUsize);

impl ck_voice::Transcriber for WordsTranscriber {
    fn transcribe(&self, wav: &std::path::Path) -> ck_core::CkResult<ck_voice::Transcript> {
        assert!(wav.is_file());
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(serde_json::from_value(json!({ "language": "en", "words": [
            { "i": 0, "text": "One", "start": 0.0, "end": 0.4 },
            { "i": 1, "text": "two", "start": 0.4, "end": 0.8 },
            { "i": 2, "text": "three", "start": 0.8, "end": 1.2 }
        ] }))
        .unwrap())
    }
}

#[test]
fn captions_follow_the_active_take_timing() {
    use crate::session::voice::{CAPTION_TRACK, CaptionsRequest, SlotRequest};

    let words = std::sync::Arc::new(WordsTranscriber::default());
    let (_dir, s) = speaking_session();
    let s = s.with_transcriber(words.clone());
    s.voice_script(&VoiceScriptRequest {
        lines: vec![line("One two three."), line("Not spoken yet.")],
        mode: None,
    })
    .unwrap();
    s.voice_speak(&SlotRequest { slot: "v1".into() }).unwrap();

    let r = s.voice_captions(&CaptionsRequest {}).unwrap();
    assert_eq!(r.captions.len(), 1);
    assert_eq!(
        (r.captions[0].slot.as_str(), r.captions[0].words),
        ("v1", 3)
    );
    assert_eq!(r.skipped, ["v2"]);
    let p = s.project().unwrap();
    assert!(p.voice_slots[0].takes[0].transcript_path.is_some());
    let track = p.tracks.iter().find(|t| t.id == CAPTION_TRACK).unwrap();
    let caption = &track.items[0];
    let voice = p.item("voice-v1").unwrap();
    assert_eq!(
        ck_core::timeline::span(&p, caption),
        ck_core::timeline::span(&p, voice)
    );

    // Again: no new transcription and no new revision.
    let again = s.voice_captions(&CaptionsRequest {}).unwrap();
    assert_eq!(again.rev, r.rev);
    assert_eq!(words.0.load(std::sync::atomic::Ordering::SeqCst), 1);
}

#[test]
fn captions_need_a_transcriber() {
    use crate::session::voice::{CaptionsRequest, SlotRequest};

    let (_dir, s) = speaking_session();
    s.voice_script(&VoiceScriptRequest {
        lines: vec![line("Hello.")],
        mode: None,
    })
    .unwrap();
    s.voice_speak(&SlotRequest { slot: "v1".into() }).unwrap();
    let err = s.voice_captions(&CaptionsRequest {}).unwrap_err();
    assert_eq!(err.code, ErrorCode::Voice);
}

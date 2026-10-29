//! The EDIT base pass through the session: transcribe (fake transcriber), packed transcript,
//! cut suggest and cut apply (spec 4, spec 8).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use ck_core::model::{Holder, ItemBody, TrackKind};
use ck_core::storage::Store;
use ck_core::{CkResult, ErrorCode};
use ck_voice::{CutKind, Transcriber, Transcript};

use super::fixture::{FakeClock, project};
use crate::session::LockRequest;
use crate::session::Session;
use crate::session::edit::{CutApplyRequest, CutWordsRequest, SourceRequest};
use crate::session::import::ImportRequest;

/// Answers with `fixtures/generated/transcript.json` (the words spoken in av.mp4).
struct FixtureTranscriber(PathBuf);

impl Transcriber for FixtureTranscriber {
    fn transcribe(&self, wav: &Path) -> CkResult<Transcript> {
        assert!(wav.is_file(), "transcribes the extracted audio");
        ck_voice::load(&self.0)
    }
}

fn fixtures() -> Option<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/generated");
    dir.join("transcript.json")
        .exists()
        .then(|| dir.canonicalize().unwrap())
}

fn session(dir: &Path, fixtures: &Path) -> Session {
    Store::new(dir).init(&project()).unwrap();
    Session::open(dir, Arc::new(FakeClock::default()))
        .unwrap()
        .with_transcriber(Arc::new(FixtureTranscriber(
            fixtures.join("transcript.json"),
        )))
}

fn src(id: &str) -> SourceRequest {
    SourceRequest { source: id.into() }
}

#[test]
fn transcribe_pack_suggest_and_apply_cuts() {
    let Some(fx) = fixtures() else {
        eprintln!("skipped: run `just fixtures` first");
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let s = session(dir.path(), &fx);
    let imported = s
        .import(&ImportRequest {
            path: fx.join("av.mp4").to_string_lossy().into_owned(),
        })
        .unwrap();
    let id = imported.source.id.clone();

    // Reading before transcribing is a clear not-found.
    assert_eq!(
        s.transcript_get(&src(&id)).unwrap_err().code,
        ErrorCode::NotFound
    );

    let t = s.transcribe(&src(&id)).unwrap();
    assert_eq!((t.rev, t.words), (2, 17));
    let stored = &s.project().unwrap().sources[0];
    assert_eq!(stored.transcript_path.as_deref(), Some(&*t.transcript_path));

    let packed = s.transcript_get(&src(&id)).unwrap();
    assert_eq!(packed.packed.fillers, 3);
    assert!(packed.packed.lines[1].starts_with("#0 1.00-2.00 Welcome (um)"));

    let suggested = s.cut_suggest(&src(&id)).unwrap();
    assert_eq!(
        suggested.duration_sec,
        imported.source.duration_sec.unwrap()
    );
    assert!(suggested.cuts.iter().any(|c| c.kind == CutKind::Retake));

    // Apply two cuts, then all: each apply replaces this source's segments.
    let two = s
        .cut_apply(&CutApplyRequest {
            source: id.clone(),
            ids: vec!["cut-1".into(), "cut-2".into()],
        })
        .unwrap();
    assert_eq!(two.track, "footage-1");
    let all = s
        .cut_apply(&CutApplyRequest {
            source: id.clone(),
            ids: vec![],
        })
        .unwrap();
    let p = s.project().unwrap();
    let track = p
        .tracks
        .iter()
        .find(|t| t.kind == TrackKind::Video)
        .unwrap();
    assert_eq!(track.items.len(), all.segments.len());
    // Segments are back to back from 0 and add up to the kept duration.
    let mut at = 0.0;
    for item in &track.items {
        let ItemBody::Footage {
            src_in,
            src_out,
            at: a,
            ..
        } = item.body
        else {
            panic!("footage expected");
        };
        assert!((a - at).abs() < 0.002, "{a} vs {at}");
        at += src_out - src_in;
    }
    assert!((at - all.duration_sec).abs() < 0.01);

    let bad = s
        .cut_apply(&CutApplyRequest {
            source: id,
            ids: vec!["cut-99".into()],
        })
        .unwrap_err();
    assert_eq!(bad.code, ErrorCode::NotFound);
}

#[test]
fn unknown_sources_and_no_transcriber_are_refused() {
    let f = super::fixture::Fixture::new();
    let err = f.session.transcribe(&src("src-nope")).unwrap_err();
    assert_eq!(err.code, ErrorCode::NotFound);
    let err = f.session.cut_suggest(&src("src-nope")).unwrap_err();
    assert_eq!(err.code, ErrorCode::NotFound);
}

#[test]
fn the_default_transcriber_needs_a_key() {
    let Some(fx) = fixtures() else {
        eprintln!("skipped: run `just fixtures` first");
        return;
    };
    let f = super::fixture::Fixture::new();
    let id = f
        .session
        .import(&ImportRequest {
            path: fx.join("tones.wav").to_string_lossy().into_owned(),
        })
        .unwrap()
        .source
        .id;
    let err = f.session.transcribe(&src(&id)).unwrap_err();
    assert_eq!(err.code, ErrorCode::Voice);
}

#[test]
fn studio_word_cuts_are_human_edits_refused_under_an_agent_track_lock() {
    let Some(fx) = fixtures() else {
        eprintln!("skipped: run `just fixtures` first");
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let s = session(dir.path(), &fx);
    let id = s
        .import(&ImportRequest {
            path: fx.join("av.mp4").to_string_lossy().into_owned(),
        })
        .unwrap()
        .source
        .id;
    s.transcribe(&src(&id)).unwrap();
    let words = |deleted: Vec<usize>| CutWordsRequest {
        source: id.clone(),
        deleted,
    };

    // Deleting "um" (word 1) splits the footage in two; the cut runs to the next word's start.
    let cut = s.cut_words(&words(vec![1])).unwrap();
    assert_eq!(cut.segments.len(), 2);
    assert!((cut.removed_sec - 0.3).abs() < 1e-9, "{}", cut.removed_sec);
    // Restoring it gives one segment over the whole source again.
    assert_eq!(s.cut_words(&words(vec![])).unwrap().segments.len(), 1);
    assert_eq!(
        s.cut_words(&words(vec![99])).unwrap_err().code,
        ErrorCode::Request
    );

    s.lock(&LockRequest {
        holder: Holder::Agent,
        layer_ids: vec!["footage-1".into()],
    })
    .unwrap();
    let refused = s.cut_words(&words(vec![1])).unwrap_err();
    assert_eq!(refused.code, ErrorCode::LockHeld);
}

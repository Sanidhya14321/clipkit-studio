//! Taste profile through the session: user picks become defaults for the next brief.

use std::sync::Arc;

use ck_core::model::{Author, BriefSource};
use ck_core::patch::PatchRequest;
use serde_json::json;

use super::fixture::{FakeClock, project};
use crate::profile::{ProfileAction, ProfileCommandRequest, run};
use crate::session::Session;
use crate::session::brief::BriefProposeRequest;

fn session(profile: &std::path::Path) -> (tempfile::TempDir, Session) {
    let dir = tempfile::tempdir().unwrap();
    ck_core::storage::Store::new(dir.path())
        .init(&project())
        .unwrap();
    let s = Session::open(dir.path(), Arc::new(FakeClock::default()))
        .unwrap()
        .with_profile(profile.to_path_buf());
    (dir, s)
}

fn pick_pacing(s: &Session, value: &str) {
    let rev = s.project().unwrap().rev;
    s.patch(&PatchRequest {
        base_rev: rev,
        layer_revs: Default::default(),
        ops: serde_json::from_value(json!([{ "op": "add", "path": "/brief/fields/pacing",
            "value": { "value": value, "source": "user", "confidence": 1.0 } }]))
        .unwrap(),
        author: Author::Human,
        label: "pick".into(),
    })
    .unwrap();
}

fn propose(s: &Session, fields: serde_json::Value) {
    s.brief_propose(&BriefProposeRequest {
        fields: serde_json::from_value(fields).unwrap(),
        ..BriefProposeRequest::default()
    })
    .unwrap();
}

#[test]
fn two_picks_become_a_profile_default_that_never_overrides() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("profile.json");
    // The user picks punchy in two projects.
    for _ in 0..2 {
        let (_d, s) = session(&file);
        pick_pacing(&s, "punchy");
    }
    let (_d, s) = session(&file);
    propose(&s, json!({ "platform": "youtube" }));
    let b = s.project().unwrap().brief;
    assert_eq!(b.fields["pacing"].value, "punchy");
    assert_eq!(b.fields["pacing"].source, BriefSource::Profile);

    // An inferred value wins over the profile.
    let (_d, s) = session(&file);
    propose(&s, json!({ "pacing": "calm" }));
    assert_eq!(s.project().unwrap().brief.fields["pacing"].value, "calm");

    // Disabled: no defaults; cleared: no picks.
    run(
        Some(&file),
        &ProfileCommandRequest {
            action: ProfileAction::Disable,
        },
    )
    .unwrap();
    let (_d, s) = session(&file);
    propose(&s, json!({ "platform": "youtube" }));
    assert!(!s.project().unwrap().brief.fields.contains_key("pacing"));
    let v = run(
        Some(&file),
        &ProfileCommandRequest {
            action: ProfileAction::Clear,
        },
    )
    .unwrap();
    assert_eq!((v.enabled, v.picks), (false, 0));
}

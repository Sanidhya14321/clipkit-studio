//! `reference analyze` with `apply`: suggestions fill the brief, user choices stay.

use ck_core::model::{Author, BriefSource};
use ck_core::patch::PatchRequest;
use serde_json::json;

use super::fixture::Fixture;
use crate::session::reference::ReferenceRequest;

#[test]
fn apply_fills_reference_fields_and_keeps_the_users() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/generated/scenes.mp4");
    if !path.is_file() {
        eprintln!("skipped: run `just fixtures` first");
        return;
    }
    let f = Fixture::new();
    let rev = f.session.project().unwrap().rev;
    f.session
        .patch(&PatchRequest {
            base_rev: rev,
            layer_revs: Default::default(),
            ops: serde_json::from_value(json!([{ "op": "add", "path": "/brief/fields/pacing",
                "value": { "value": "calm", "source": "user", "confidence": 1.0 } }]))
            .unwrap(),
            author: Author::Human,
            label: "user pacing".into(),
        })
        .unwrap();
    let r = f
        .session
        .reference_analyze(&ReferenceRequest {
            path: path.to_string_lossy().into_owned(),
            apply: true,
        })
        .unwrap();
    assert_eq!(r.suggested["pacing"], "lively");
    assert_eq!(r.applied, ["length", "aspect"]);
    let b = f.session.project().unwrap().brief;
    assert_eq!(b.fields["pacing"].value, "calm");
    assert_eq!(b.fields["aspect"].source, BriefSource::Reference);
    assert_eq!(b.fields["length"].value, 10.0);
}

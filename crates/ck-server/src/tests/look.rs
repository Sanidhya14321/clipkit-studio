//! `look apply` through the client: re-theme by one token write, impact preview, locks
//! (spec 8.1).

use ck_core::ErrorCode;
use ck_core::model::{Holder, ItemBody};
use ck_core::patch::PROJECT_LAYER;
use serde_json::{Value, json};

use super::ops::Live;
use crate::session::LockRequest;
use crate::session::look::LookApplyRequest;
use crate::session::recipe::RecipeApplyRequest;

fn look(id: &str, dry_run: bool) -> LookApplyRequest {
    LookApplyRequest {
        look: id.into(),
        dry_run,
    }
}

fn hook() -> RecipeApplyRequest {
    serde_json::from_value(json!({ "recipe": "hook-title", "slots": { "headline": "Hi" } }))
        .unwrap()
}

fn props(live: &Live) -> Vec<Value> {
    let p = live.f.session.project().unwrap();
    p.tracks
        .iter()
        .flat_map(|t| &t.items)
        .filter_map(|i| match &i.body {
            ItemBody::Clip { props, .. } => Some(props.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn apply_rethemes_with_one_write_and_previews_the_impact() {
    let live = Live::start();
    let ops = live.ops();
    ops.recipe_apply(&hook()).unwrap();
    let before = props(&live);

    let preview = ops.look_apply(&look("cinematic", true)).unwrap();
    assert_eq!((preview.rev, preview.applied), (1, false));
    assert_eq!(preview.previous, "clean-product");
    assert_eq!(
        preview.impacted,
        ["hook-title-headline", "hook-title-backdrop"],
        "track order"
    );
    assert_eq!(
        live.f.session.project().unwrap().rev,
        1,
        "dry run writes nothing"
    );

    let r = ops.look_apply(&look("cinematic", false)).unwrap();
    assert_eq!((r.rev, r.applied), (2, true));
    assert_eq!(r.impacted, preview.impacted);
    let p = live.f.session.project().unwrap();
    assert_eq!(p.tokens["look"], "cinematic");
    assert_eq!(props(&live), before, "layers keep their token refs");
    assert!(ops.status().unwrap().locks.is_empty(), "lock released");

    // Same look again: nothing to write and nothing changes.
    let again = ops.look_apply(&look("cinematic", false)).unwrap();
    assert_eq!((again.rev, again.applied), (2, false));
    assert!(again.impacted.is_empty());

    // The stored look wins over the brief for recipe apply and verify.
    ops.brief_propose(
        &serde_json::from_value(json!({ "fields": { "look": "bold-social" } })).unwrap(),
    )
    .unwrap();
    assert_eq!(ops.recipe_apply(&hook()).unwrap().look, "cinematic");
    assert_eq!(ops.verify(&Default::default()).unwrap().look, "cinematic");
}

#[test]
fn apply_refuses_unknown_looks_and_human_locks() {
    let live = Live::start();
    let ops = live.ops();
    assert_eq!(
        ops.look_apply(&look("nope", false)).unwrap_err().code,
        ErrorCode::NotFound
    );
    ops.lock(&LockRequest {
        holder: Holder::Human,
        layer_ids: vec![PROJECT_LAYER.into()],
    })
    .unwrap();
    assert_eq!(
        ops.look_apply(&look("cinematic", false)).unwrap_err().code,
        ErrorCode::LockHeld
    );
    assert_eq!(live.f.session.project().unwrap().rev, 0);
    // A preview needs no lock.
    assert!(!ops.look_apply(&look("cinematic", true)).unwrap().applied);
}

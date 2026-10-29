//! `recipe apply` through the client: named layers, tracks, provenance, locks (spec 7.1, 7.5).

use ck_core::ErrorCode;
use ck_core::model::{Holder, ItemBody, TrackKind};
use ck_core::patch::PROJECT_LAYER;
use serde_json::{Value, json};

use super::ops::Live;
use crate::session::LockRequest;
use crate::session::recipe::RecipeApplyRequest;

fn apply(v: Value) -> RecipeApplyRequest {
    serde_json::from_value(v).unwrap()
}

fn hook(headline: &str) -> RecipeApplyRequest {
    apply(json!({ "recipe": "hook-title", "at": 1.0, "slots": { "headline": headline } }))
}

#[test]
fn apply_adds_named_layers_with_provenance_and_tokens() {
    let live = Live::start();
    let ops = live.ops();

    let r = ops.recipe_apply(&hook("Ship video in minutes")).unwrap();
    assert_eq!(r.rev, 1);
    assert_eq!(r.look, "clean-product", "no brief look: the default");
    assert_eq!(r.layers, ["hook-title-backdrop", "hook-title-headline"]);
    assert!(
        ops.status().unwrap().locks.is_empty(),
        "temporary locks are released"
    );

    let p = live.f.session.project().unwrap();
    // The fixture has an overlay track; a video track is added for the backdrop.
    let kinds: Vec<(&str, TrackKind)> = p.tracks.iter().map(|t| (t.id.as_str(), t.kind)).collect();
    assert_eq!(
        kinds,
        [("t1", TrackKind::Overlay), ("video-1", TrackKind::Video)]
    );
    assert_eq!(p.assets_used, ["recipe:hook-title"]);
    let head = p.item("hook-title-headline").unwrap();
    assert_eq!(head.brief_deps, ["goal", "pacing", "look"]);
    let prov = head.provenance.as_ref().unwrap();
    assert_eq!(prov.source_asset, "recipe:hook-title");
    assert_eq!(prov.adaptations, ["headline"]);
    let ItemBody::Clip { at, props, .. } = &head.body else {
        panic!("clip expected");
    };
    assert_eq!(*at, 1.1);
    assert_eq!(props["text"], "Ship video in minutes");
    assert_eq!(props["color"], "$colors.text", "tokens stay references");
    assert!(props.get("kicker").is_none(), "optional slot left out");

    // Applying again picks the next free prefix and reuses the tracks.
    let again = ops.recipe_apply(&hook("Again")).unwrap();
    assert_eq!(again.layers[0], "hook-title-2-backdrop");
    assert_eq!(live.f.session.project().unwrap().tracks.len(), 2);
}

#[test]
fn apply_uses_the_brief_look_and_refuses_bad_input() {
    let live = Live::start();
    let ops = live.ops();
    ops.brief_propose(
        &serde_json::from_value(json!({ "fields": { "look": "cinematic" } })).unwrap(),
    )
    .unwrap();
    assert_eq!(ops.recipe_apply(&hook("Hi")).unwrap().look, "cinematic");

    let code = |req: RecipeApplyRequest| ops.recipe_apply(&req).unwrap_err().code;
    assert_eq!(
        code(apply(json!({ "recipe": "nope" }))),
        ErrorCode::NotFound
    );
    assert_eq!(
        code(apply(json!({ "recipe": "hook-title", "slots": {} }))),
        ErrorCode::Request
    );
    assert_eq!(
        code(apply(
            json!({ "recipe": "hook-title", "slots": { "headline": "x" }, "look": "nope" })
        )),
        ErrorCode::NotFound
    );
    assert_eq!(
        code(apply(
            json!({ "recipe": "hook-title", "slots": { "headline": "x" }, "prefix": "hook-title" })
        )),
        ErrorCode::Request,
        "a taken prefix is refused"
    );
    assert_eq!(
        code(apply(
            json!({ "recipe": "talking-head", "slots": { "media": "missing" } })
        )),
        ErrorCode::Request,
        "a source slot needs a project source"
    );
}

#[test]
fn apply_respects_a_human_lock() {
    let live = Live::start();
    let ops = live.ops();
    ops.lock(&LockRequest {
        holder: Holder::Human,
        // `assetsUsed` is project content, so the write touches the project lock.
        layer_ids: vec![PROJECT_LAYER.into()],
    })
    .unwrap();
    let e = ops.recipe_apply(&hook("Hi")).unwrap_err();
    assert_eq!(e.code, ErrorCode::LockHeld);
    assert_eq!(live.f.session.project().unwrap().rev, 0);
}

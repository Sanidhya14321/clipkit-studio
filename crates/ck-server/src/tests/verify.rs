//! `verify` (spec 4.3): each gate passes a clean generated project and catches its own fault.

use ck_assets::library::{self, IdRequest};
use ck_assets::look::Look;
use ck_core::Project;
use ck_core::model::Mode;
use serde_json::{Value, json};

use super::ops::Live;
use crate::session::recipe::RecipeApplyRequest;
use crate::session::verify::{GATES, MAX_REUSE, VerifyResult, verify};

fn look() -> Look {
    library::look_get(&IdRequest {
        id: "clean-product".into(),
    })
    .unwrap()
}

fn prov(asset: &str) -> Value {
    json!({ "sourceAsset": asset, "hash": "ab12", "mode": "recipe", "adaptations": [], "license": "original" })
}

fn backdrop(id: &str, at: f64, dur: f64, color: &str) -> Value {
    json!({ "id": id, "name": "Scene backdrop", "type": "clip", "clipId": "backdrop", "at": at,
            "dur": dur, "props": { "color": color }, "provenance": prov("recipe:hook-title") })
}

fn text(id: &str, at: f64, dur: f64, props: Value) -> Value {
    let mut p = json!({ "text": "Ship video in minutes", "color": "$colors.text", "font": "$fonts.display" });
    p.as_object_mut()
        .unwrap()
        .extend(props.as_object().unwrap().clone());
    json!({ "id": id, "name": "Scene headline", "type": "clip", "clipId": "kinetic-text", "at": at,
            "dur": dur, "props": p, "provenance": prov("recipe:hook-title") })
}

/// A 4 s generated project: one backdrop and one headline from the hook-title recipe.
fn clean() -> Project {
    let mut p = Project::new("p1", "Demo", Mode::Generate);
    p.tracks = serde_json::from_value(json!([
        { "id": "video-1", "kind": "video", "items": [backdrop("bg", 0.0, 4.0, "$colors.background")] },
        { "id": "overlay-1", "kind": "overlay", "items": [text("head", 0.3, 3.7, json!({}))] }
    ]))
    .unwrap();
    p.assets_used = vec!["recipe:hook-title".into()];
    p.brief.fields = serde_json::from_value(json!({
        "length": { "value": 4, "source": "user", "confidence": 1.0 },
        "platform": { "value": "landscape", "source": "user", "confidence": 1.0 }
    }))
    .unwrap();
    p
}

fn push(p: &mut Project, track: usize, item: Value) {
    p.tracks[track]
        .items
        .push(serde_json::from_value(item).unwrap());
}

fn failed(r: &VerifyResult) -> Vec<&str> {
    r.failed.iter().map(String::as_str).collect()
}

#[test]
fn clean_project_passes_every_gate() {
    let r = verify(&clean(), &look());
    assert!(r.ok, "{:?}", r.issues);
    assert_eq!(r.passed, GATES);
    assert_eq!(r.look, "clean-product");
}

#[test]
fn timeline_gates_catch_gaps_overlaps_and_duration() {
    let mut p = clean();
    p.tracks[0].items[0] = serde_json::from_value(backdrop("bg", 0.5, 3.5, "#ffffff")).unwrap();
    assert_eq!(failed(&verify(&p, &look())), ["gaps"]);

    let mut p = clean();
    push(&mut p, 0, backdrop("bg2", 3.0, 1.0, "$colors.background"));
    let r = verify(&p, &look());
    assert_eq!(failed(&r), ["overlaps"]);
    assert_eq!(r.issues[0].ids, ["bg", "bg2"]);

    let mut p = clean();
    p.brief.fields.get_mut("length").unwrap().value = json!(15);
    assert_eq!(failed(&verify(&p, &look())), ["duration"]);
}

#[test]
fn schema_gate_catches_unknown_clips_and_bad_refs() {
    let mut p = clean();
    push(
        &mut p,
        1,
        json!({ "id": "x", "name": "x", "type": "clip", "clipId": "nope", "at": 0.0, "dur": 1.0,
                "provenance": prov("recipe:hook-title") }),
    );
    assert_eq!(failed(&verify(&p, &look())), ["schema"]);

    let mut p = clean();
    push(
        &mut p,
        1,
        json!({ "id": "v", "type": "voice", "slotId": "missing", "at": 0.0 }),
    );
    assert!(failed(&verify(&p, &look())).contains(&"schema"));
}

#[test]
fn text_gates_catch_layout_contrast_and_fonts() {
    // Off the safe area.
    let mut p = clean();
    p.tracks[1].items[0] = serde_json::from_value(text(
        "head",
        0.3,
        3.7,
        json!({ "box": { "x": 0.0, "y": 0.0, "w": 1.0, "h": 0.1 }, "fontSize": 0.06,
                "lines": ["Ship video in minutes"] }),
    ))
    .unwrap();
    assert_eq!(failed(&verify(&p, &look())), ["layout"]);

    // Light grey text on a white backdrop.
    let mut p = clean();
    p.tracks[0].items[0] = serde_json::from_value(backdrop("bg", 0.0, 4.0, "#ffffff")).unwrap();
    p.tracks[1].items[0] =
        serde_json::from_value(text("head", 0.3, 3.7, json!({ "color": "#dddddd" }))).unwrap();
    let r = verify(&p, &look());
    assert_eq!(failed(&r), ["contrast"]);
    assert_eq!(r.issues[0].ids, ["head"]);

    // Three families (the look's display font plus two hard-coded ones), each on its own time.
    let mut p = clean();
    push(
        &mut p,
        1,
        text("b", 0.3, 0.1, json!({ "font": "Comic Sans" })),
    );
    push(&mut p, 1, text("c", 0.3, 0.1, json!({ "font": "Papyrus" })));
    let r = verify(&p, &look());
    assert!(failed(&r).contains(&"fonts"), "{:?}", r.issues);
}

#[test]
fn motion_gate_allows_three_animated_layers() {
    let mut p = clean();
    // Untexted clips so only motion counts; the backdrop does not animate.
    let clip = |id: &str| {
        json!({ "id": id, "name": id, "type": "clip", "clipId": "device-frame", "at": 1.0,
                "dur": 1.0, "provenance": prov("recipe:hook-title") })
    };
    push(&mut p, 1, clip("m1"));
    push(&mut p, 1, clip("m2"));
    assert!(verify(&p, &look()).ok, "3 animated layers are allowed");
    push(&mut p, 1, clip("m3"));
    let r = verify(&p, &look());
    assert_eq!(failed(&r), ["motion"]);
    assert_eq!(r.issues[0].ids.len(), 4);
}

#[test]
fn reuse_gate_counts_applications_and_honours_motif() {
    let mut p = clean();
    p.brief.fields.remove("length");
    let mut at = 4.0;
    for n in 0..MAX_REUSE {
        push(
            &mut p,
            0,
            backdrop(&format!("bg{n}"), at, 1.0, "$colors.background"),
        );
        at += 1.0;
    }
    let r = verify(&p, &look());
    assert_eq!(failed(&r), ["reuse"]);
    assert_eq!(r.issues[0].ids.len(), MAX_REUSE + 1);

    for item in &mut p.tracks[0].items {
        if let ck_core::model::ItemBody::Clip { props, .. } = &mut item.body {
            props["motif"] = json!(true);
        }
    }
    assert!(verify(&p, &look()).ok);
}

#[test]
fn provenance_gate_needs_trace_listing_and_reference() {
    let mut p = clean();
    p.assets_used.push("recipe:unused".into());
    assert_eq!(failed(&verify(&p, &look())), ["provenance"]);

    let mut p = clean();
    p.tracks[1].items[0].provenance = None;
    assert_eq!(failed(&verify(&p, &look())), ["provenance"]);

    let mut p = clean();
    p.assets_used.clear();
    assert_eq!(failed(&verify(&p, &look())), ["provenance"]);
}

#[test]
fn placeholder_gate_catches_unfilled_slots() {
    let mut p = clean();
    p.tracks[1].items[0] =
        serde_json::from_value(text("head", 0.3, 3.7, json!({ "kicker": "{kicker}" }))).unwrap();
    let r = verify(&p, &look());
    assert_eq!(failed(&r), ["placeholders"]);
}

#[test]
fn verify_through_the_client_after_recipe_apply() {
    let live = Live::start();
    let ops = live.ops();
    // The fixture's untraced `title` clip has no clip id, so it needs no provenance; its 0 to
    // 2 s span sits on an overlay track.
    ops.brief_propose(
        &serde_json::from_value(json!({ "fields": { "platform": "landscape", "length": 3 } }))
            .unwrap(),
    )
    .unwrap();
    let req: RecipeApplyRequest = serde_json::from_value(json!({
        "recipe": "hook-title", "at": 0.0, "duration": 3.0,
        "slots": { "headline": "Ship video in minutes" }
    }))
    .unwrap();
    ops.recipe_apply(&req).unwrap();
    let r = ops.verify(&Default::default()).unwrap();
    assert!(r.ok, "{:?}", r.issues);
    assert_eq!(r.rev, 2);
}

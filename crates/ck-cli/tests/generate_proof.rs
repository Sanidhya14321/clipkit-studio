//! Phase 4 proof (spec 12): prompt to brief to a 15 second generated video, through the real
//! `ck` binary against a real `ck studio`, then exported through the render worker (step 10b,
//! built in Phase 9).
#![allow(clippy::unwrap_used)] // test helpers outside #[test] functions

mod common;

use std::path::Path;

use ck_server::ops::caps;
use common::{Studio, ok};
use serde_json::{Value, json};

/// Write `req` to a file and run `ck <args> <file>`.
fn with_file(d: &Path, name: &str, args: &[&str], req: &Value, cap: usize) -> Value {
    let file = d.join(name);
    std::fs::write(&file, req.to_string()).unwrap();
    let mut all = args.to_vec();
    all.push(file.to_str().unwrap());
    ok(d, &all, cap)
}

#[test]
fn prompt_to_brief_to_a_verified_15_second_generated_video() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _studio = Studio::start(d);

    // The agent proposes the brief it inferred from the prompt; the human confirms it.
    let proposal = json!({
        "fields": { "platform": "youtube", "length": 15, "look": "clean-product" },
        "questions": []
    });
    let proposed = with_file(
        d,
        "brief.json",
        &["brief", "propose"],
        &proposal,
        caps::BRIEF_PROPOSE,
    );
    let rev = proposed["rev"].as_u64().unwrap();
    let confirm = json!({
        "baseRev": rev, "author": "human",
        "ops": [{ "op": "replace", "path": "/brief/status", "value": "confirmed" }]
    });
    ok(d, &["project", "patch", &confirm.to_string()], caps::PATCH);
    let ready = ok(d, &["brief", "wait", "--timeout", "0"], caps::BRIEF);
    assert_eq!(ready["ready"], true, "{ready}");

    // Three scenes back to back fill the 15 seconds.
    let scenes = [
        json!({ "recipe": "hook-title", "at": 0.0, "duration": 4.0,
                "slots": { "headline": "Ship video in minutes" } }),
        json!({ "recipe": "stat-callout", "at": 4.0, "duration": 5.0,
                "slots": { "value": 87, "unit": "%", "label": "less editing time" } }),
        json!({ "recipe": "outro-cta", "at": 9.0, "duration": 6.0,
                "slots": { "action": "Try it free", "handle": "clipkit.dev" } }),
    ];
    for (i, scene) in scenes.iter().enumerate() {
        let applied = with_file(
            d,
            &format!("scene{i}.json"),
            &["recipe", "apply"],
            scene,
            caps::RECIPE_APPLY,
        );
        assert!(
            !applied["layers"].as_array().unwrap().is_empty(),
            "{applied}"
        );
    }

    let layout = ok(d, &["layout", "check"], caps::LAYOUT_CHECK);
    assert_eq!(layout["ok"], true, "{layout}");
    let verified = ok(d, &["verify"], caps::VERIFY);
    assert_eq!(verified["ok"], true, "every gate passes: {verified}");

    let sound = ok(d, &["sound", "plan"], caps::SOUND_PLAN);
    assert!(!sound["cues"].as_array().unwrap().is_empty(), "{sound}");

    common::to_verify(d);
    let handed = ok(d, &["handoff"], caps::HANDOFF);
    let waited = ok(
        d,
        &[
            "wait",
            "--since",
            &handed["rev"].to_string(),
            "--timeout",
            "0",
        ],
        caps::WAIT,
    );
    assert_eq!(waited["events"], json!([]));

    // Step 10b (Phase 9): export through the render worker. Skipped where the worker is not
    // bundled (`just worker`), like the fixture-backed tests.
    let worker =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../render-worker/bundle/index.html");
    if !worker.is_file() {
        eprintln!("skipped export: run `just worker` first");
        return;
    }
    let exported = ok(d, &["export"], caps::EXPORT);
    assert_eq!(exported["via"], "render", "{exported}");
    let file = d.join(exported["path"].as_str().unwrap());
    let info = ck_media::probe(&file).unwrap();
    assert!((info.duration - 15.0).abs() < 0.2, "{}", info.duration);
    assert_eq!(info.audio.len(), 1);
    // Determinism: each frame rendered twice, out of order, is identical.
    let check = ok(d, &["render", "check"], caps::RENDER_CHECK);
    assert_eq!(check["deterministic"], true, "{check}");

    // The critic's contact sheet: 6 frames tiled 3x2 at 320 px.
    let critic = ok(d, &["verify", "--critic"], caps::VERIFY);
    let sheet = d.join(critic["contactSheet"].as_str().unwrap());
    let v = ck_media::probe(&sheet).unwrap().video.unwrap();
    assert_eq!(v.width, 960, "{critic}");

    // Variants: one still per look at the middle frame.
    let v = ok(
        d,
        &["variants", "--look", "cinematic", "--look", "bold-social"],
        ck_server::session::branch::caps::VARIANTS,
    );
    assert_eq!(v["stills"].as_array().unwrap().len(), 2, "{v}");
    assert!(
        d.join(format!(".clipkit/variants/cinematic-{}.png", v["frame"]))
            .is_file()
    );

    // The same project again comes from the render cache.
    let t = std::time::Instant::now();
    ok(d, &["export", "--out", "exports/again.mp4"], caps::EXPORT);
    assert!(t.elapsed() < std::time::Duration::from_secs(20));
}

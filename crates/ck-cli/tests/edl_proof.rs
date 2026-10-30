//! The Edit Decision Plan, through the real binary: grid, segments, expansion, scoped change,
//! gates and export, on the fixture clip and on a generated 9:16 brief.
#![allow(clippy::unwrap_used)] // test helpers outside #[test] functions

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use common::Studio;
use serde_json::{Value, json};

fn raw(d: &Path, args: &[&str]) -> Value {
    let out = Command::new(env!("CARGO_BIN_EXE_ck"))
        .args(args)
        .current_dir(d)
        .output()
        .unwrap();
    serde_json::from_slice(&[out.stdout, out.stderr].concat()).unwrap()
}

fn project(d: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(d.join(".clipkit/project.json")).unwrap())
        .unwrap()
}

fn layer(p: &Value, id: &str) -> Value {
    p["tracks"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|t| t["items"].as_array().unwrap())
        .find(|i| i["id"] == id)
        .cloned()
        .unwrap_or(Value::Null)
}

fn fixtures() -> Option<PathBuf> {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/generated");
    p.join("av.mp4").is_file().then_some(p)
}

fn worker() -> bool {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../render-worker/bundle/index.html")
        .is_file()
}

fn seg(id: &str, from: usize, to: usize, role: &str, headline: &str) -> Value {
    json!({ "id": id, "from": from, "to": to, "role": role, "recipe": "section-card",
            "slots": { "title": headline } })
}

#[test]
fn generate_on_a_fast_grid_in_nine_by_sixteen_to_export() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _s = Studio::start(d);
    let brief = json!({ "fields": { "aspect": "9:16" }, "questions": [] }).to_string();
    raw(d, &["brief", "propose", &brief]);

    let grid = raw(d, &["grid", "build", "--dur", "6"]);
    assert_eq!(grid["tempo"], 120.0, "fast by default: {grid}");
    assert_eq!(grid["anchors"].as_array().unwrap().len(), 13, "{grid}");

    let hook = json!({ "id": "open", "from": 0, "to": 4, "role": "hook",
        "recipe": "hook-title", "slots": { "headline": "Ship faster" } });
    let cta = json!({ "id": "close", "from": 4, "to": 12, "role": "cta",
        "recipe": "hook-title", "slots": { "headline": "Try it now" } });
    let plan = json!({ "segments": [hook, cta] }).to_string();
    assert_eq!(raw(d, &["plan", "set", &plan])["segments"], 2);

    let picks = raw(d, &["plan", "candidates"]);
    let first = picks["lines"][0].as_str().unwrap();
    assert!(first.starts_with("open Hook: "), "{picks}");
    assert!(first.matches('|').count() <= 2, "at most 3: {picks}");

    let out = raw(d, &["plan", "expand"]);
    assert!(out["layers"].as_u64().unwrap() >= 4, "{out}");
    let p = project(d);
    assert_eq!(p["plan"]["edl"]["segments"].as_array().unwrap().len(), 2);
    assert_eq!(layer(&p, "edl-close-backdrop")["at"].as_f64(), Some(2.0));

    // A scoped change touches one segment's layers only.
    let open_rev = layer(&p, "edl-open-headline")["rev"].clone();
    let again = raw(d, &["plan", "expand", "--segment", "close"]);
    assert_eq!(again["segments"], 1, "{again}");
    let p = project(d);
    assert_eq!(layer(&p, "edl-open-headline")["rev"], open_rev);

    if worker() {
        let done = raw(
            d,
            &["export", "--force", "--reason", "plan proof, no audio"],
        );
        let path = d.join(done["path"].as_str().expect("export path"));
        let info = Command::new("ffprobe")
            .args(["-v", "error", "-select_streams", "v:0"])
            .args(["-show_entries", "stream=width,height", "-of", "csv=p=0"])
            .arg(&path)
            .output()
            .unwrap();
        assert_eq!(String::from_utf8_lossy(&info.stdout).trim(), "1080,1920");
    }
}

#[test]
fn bad_plans_are_refused_with_a_hint() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _s = Studio::start(d);
    assert_eq!(raw(d, &["plan", "get"])["code"], "E_NOT_FOUND");
    raw(d, &["grid", "build", "--dur", "4", "--bpm", "60"]);
    let bad = json!({ "segments": [seg("a", 0, 99, "point", "x")] }).to_string();
    let e = raw(d, &["plan", "set", &bad]);
    assert_eq!(e["code"], "E_REQUEST", "{e}");
    assert!(!e["hint"].as_str().unwrap().is_empty());
}

#[test]
fn speech_clip_keeps_phrases_with_captions_on_exact_words() {
    let Some(fx) = fixtures() else {
        eprintln!("skipped: run `just fixtures` first");
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _s = Studio::start(d);
    let imported = raw(d, &["import", fx.join("av.mp4").to_str().unwrap()]);
    let id = imported["source"]["id"].as_str().unwrap().to_owned();
    let hash = imported["source"]["hash"].as_str().unwrap().to_owned();
    let rel = format!(".clipkit/cache/transcripts/{hash}.json");
    std::fs::create_dir_all(d.join(".clipkit/cache/transcripts")).unwrap();
    std::fs::copy(fx.join("transcript.json"), d.join(&rel)).unwrap();
    let attach = json!({ "baseRev": imported["rev"], "author": "human",
        "ops": [{ "op": "add", "path": "/sources/0/transcriptPath", "value": rel }] });
    raw(d, &["project", "patch", &attach.to_string()]);

    let map = raw(d, &["content", "map", &id]);
    assert!(map["words"].as_u64().unwrap() > 0, "{map}");
    let grid = raw(d, &["grid", "build", "--source", &id]);
    assert_eq!(grid["source"], "speech", "{grid}");
    let last = grid["anchors"].as_array().unwrap().len() - 1;
    let plan = json!({ "segments": [{ "id": "all", "from": 0, "to": last, "role": "point",
        "source": id, "captions": true }] });
    assert_eq!(raw(d, &["plan", "set", &plan.to_string()])["segments"], 1);
    let out = raw(d, &["plan", "expand"]);
    assert!(out["layers"].as_u64().unwrap() > 0, "{out}");

    let p = project(d);
    assert!(!layer(&p, &format!("{id}-seg-1")).is_null(), "footage laid");
    let cap = layer(&p, "edl-all-cap-1");
    assert_eq!(cap["clipId"], "kinetic-text", "{cap}");
    assert!(cap["props"]["box"].is_object(), "{cap}");
    assert!(cap["props"]["text"].as_str().unwrap().split(' ').count() <= 3);
    let check = raw(d, &["layout", "check"]);
    assert_eq!(check["ok"], true, "no collisions: {check}");
}

#[test]
fn queued_export_returns_at_once_and_reports_its_end() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _s = Studio::start(d);
    // Nothing to export: the job must end failed, with the reason, not hang.
    let job = raw(d, &["export", "--queue"]);
    assert_eq!(job["id"], "export-1", "{job}");
    for _ in 0..100 {
        let jobs = raw(d, &["jobs"]);
        if jobs["jobs"][0]["state"] == "failed" {
            assert!(
                jobs["jobs"][0]["error"]
                    .as_str()
                    .unwrap()
                    .contains("no layers"),
                "{jobs}"
            );
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    panic!("job never finished");
}

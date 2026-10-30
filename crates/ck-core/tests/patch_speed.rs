//! Patch latency on a large project (spec 2: a CLI patch shows in the browser in under 300 ms).
//! Ignored by default; run with `cargo test --release -p ck-core --test patch_speed -- --ignored
//! --nocapture` to print timings.
#![allow(clippy::unwrap_used)]

use std::time::Instant;

use ck_core::model::{Author, Mode};
use ck_core::patch::PatchRequest;
use ck_core::{Document, Project};
use serde_json::json;

fn big(n: usize) -> Project {
    let mut p = Project::new("p", "Big", Mode::Generate);
    let items: Vec<_> = (0..n)
        .map(|i| {
            json!({ "id": format!("c{i}"), "name": format!("Clip {i}"), "type": "clip",
                         "at": i as f64 * 0.5, "dur": 0.5,
                         "props": { "text": "Some headline text", "color": "$color.text" } })
        })
        .collect();
    p.tracks =
        serde_json::from_value(json!([{ "id": "t", "kind": "overlay", "items": items }])).unwrap();
    p
}

#[test]
#[ignore = "timing; run in release by hand"]
fn patch_latency_on_large_projects() {
    for n in [100, 500, 2000] {
        let mut doc = Document::new(big(n));
        let start = Instant::now();
        let runs = 50;
        for k in 0..runs {
            let req = PatchRequest {
                base_rev: doc.project().rev,
                layer_revs: Default::default(),
                ops: serde_json::from_value(json!([{ "op": "replace",
                    "path": "/tracks/0/items/0/name", "value": format!("n{k}") }]))
                .unwrap(),
                author: Author::Human,
                label: "rename".into(),
            };
            doc.patch(&req, k).unwrap();
        }
        let per = start.elapsed().as_secs_f64() * 1000.0 / runs as f64;
        println!("{n} items: {per:.2} ms per patch");
    }
}

//! Phase 8 proof (spec 12): sources are registered, vetted without running their code and
//! ingested into cards; search returns them within the budget gate in the right license mode;
//! a restricted fixture is blocked in commercial-safe mode and a hostile one never appears; a
//! derived copy carries tokens and instrumentation; the license report lists it. Through the
//! real `ck` binary against a real `ck studio`.
#![allow(clippy::unwrap_used)] // test helpers outside #[test] functions

mod common;

use std::path::Path;

use ck_assets::{ingest, report, retrieve, sources, vet};
use ck_server::ops::caps;
use common::{Studio, err, ok};
use serde_json::json;

fn pack(name: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/sources")
        .join(name)
        .canonicalize()
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

#[test]
fn sources_become_searchable_cards_within_budget_and_license() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    std::fs::write(d.join("sources.yaml"), "[]").unwrap();
    for (id, from) in [
        ("mit", "pack-mit"),
        ("restricted", "pack-restricted"),
        ("hostile", "pack-injection"),
    ] {
        ok(
            d,
            &["sources", "add", id, "--kind", "remotion-components"],
            sources::caps::SOURCES_WRITE,
        );
        let vetted = ok(
            d,
            &["sources", "vet", id, "--from", &pack(from)],
            vet::caps::VET,
        );
        let want = if id == "hostile" {
            "rejected"
        } else {
            "vetted"
        };
        assert_eq!(vetted["status"], want, "{id}: {vetted}");
    }
    ok(d, &["sources", "ingest", "mit"], ingest::caps::INGEST);
    ok(
        d,
        &["sources", "ingest", "restricted"],
        ingest::caps::INGEST,
    );
    assert_eq!(
        err(d, &["sources", "ingest", "hostile"])["code"],
        "E_REQUEST"
    );
    let gaps = ok(d, &["sources", "gaps"], sources::caps::SOURCES_GAPS);
    assert_eq!((&gaps["usable"], &gaps["rejected"]), (&json!(2), &json!(1)));

    // A project with a confirmed commercial-safe brief.
    let _studio = Studio::start(d);
    let status = ok(d, &["status"], caps::STATUS);
    let brief = json!({
        "baseRev": status["rev"], "author": "human",
        "ops": [{ "op": "add", "path": "/brief/licenseMode", "value": "commercial-safe" }]
    });
    ok(d, &["project", "patch", &brief.to_string()], caps::PATCH);

    let hits = ok(
        d,
        &["assets", "search", "ticker title glow"],
        retrieve::caps::SEARCH,
    );
    let lines = hits["hits"].to_string();
    assert!(lines.contains("mit/glow-title"), "{lines}");
    assert!(
        !lines.contains("news-ticker") && !lines.contains("hostile"),
        "{lines}"
    );
    let personal = ok(
        d,
        &["assets", "search", "ticker", "--mode", "personal"],
        retrieve::caps::SEARCH,
    );
    assert!(
        personal["hits"][0]
            .as_str()
            .unwrap()
            .starts_with("restricted/news-ticker")
    );

    // The ladder: two cards, a skeleton, a slice; the third card is over budget.
    ok(
        d,
        &["assets", "card", "mit/glow-title"],
        retrieve::caps::CARD,
    );
    ok(d, &["assets", "card", "mit/slide-in"], retrieve::caps::CARD);
    let over = err(d, &["assets", "card", "restricted/news-ticker"]);
    assert_eq!(over["code"], "E_BUDGET");
    let sk = ok(
        d,
        &["assets", "skeleton", "mit/glow-title"],
        retrieve::caps::SKELETON,
    );
    assert!(
        sk["text"]
            .as_str()
            .unwrap()
            .contains("props GlowTitleProps")
    );
    assert!(sk["budget"]["used"].as_u64().unwrap() <= retrieve::TOKEN_TARGET);

    // Derive: tokens and instrumentation; a restricted asset is refused in this mode.
    let derived = ok(
        d,
        &["assets", "derive", "mit/glow-title", "--as", "glow"],
        caps::DERIVE,
    );
    assert!(
        derived["adaptations"].to_string().contains("instrumented"),
        "{derived}"
    );
    let blocked = err(
        d,
        &[
            "assets",
            "derive",
            "restricted/news-ticker",
            "--as",
            "ticker",
        ],
    );
    assert_eq!(blocked["code"], "E_REQUEST");
    let src = std::fs::read_to_string(d.join(".clipkit/derived/glow/GlowTitle.tsx")).unwrap();
    assert!(src.contains("ckTokens.color1") && src.contains("data-ck-layer={ckLayerId}"));

    let report = ok(d, &["license", "report"], report::caps::REPORT);
    assert_eq!(report["ok"], true, "{report}");
    assert_eq!(report["assets"][0]["spdx"], "MIT");
    assert!(
        report["runtime"]
            .as_str()
            .unwrap()
            .contains("Remotion License")
    );
}

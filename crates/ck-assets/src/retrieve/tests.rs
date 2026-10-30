use super::*;
use crate::ingest::tests::ingested_hub;

/// A project folder whose `.clipkit/project.json` has `rev` and an optional license mode.
fn project(rev: u64, mode: Option<&str>) -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(d.path().join(".clipkit")).unwrap();
    let brief = match mode {
        Some(m) => serde_json::json!({ "licenseMode": m }),
        None => serde_json::json!({}),
    };
    std::fs::write(
        d.path().join(".clipkit/project.json"),
        serde_json::json!({ "rev": rev, "brief": brief }).to_string(),
    )
    .unwrap();
    d
}

fn search(r: &Retrieval, q: &str, mode: Option<LicenseMode>) -> Vec<String> {
    r.search(&AssetSearchRequest {
        query: q.into(),
        mode,
        ..Default::default()
    })
    .unwrap()
    .hits
}

#[test]
fn search_ranks_and_respects_the_license_mode() {
    let (_h, hub) = ingested_hub();
    let p = project(1, None);
    let r = Retrieval::new(hub, p.path());
    let hits = search(&r, "glow title", None);
    assert!(
        hits[0].starts_with("pack-mit/glow-title remotion-clip A permissive:"),
        "{hits:?}"
    );
    // The restricted ticker is hidden in the default commercial-safe mode, shown in personal.
    assert!(
        !search(&r, "ticker", None)
            .iter()
            .any(|h| h.contains("news-ticker"))
    );
    assert!(
        search(&r, "ticker", Some(LicenseMode::Personal))[0]
            .starts_with("pack-restricted/news-ticker")
    );
    // Nothing from the rejected hostile pack, whatever the mode.
    assert!(
        !search(&r, "sparkle", Some(LicenseMode::Any))
            .iter()
            .any(|h| h.contains("injection"))
    );
}

#[test]
fn the_brief_license_mode_is_the_default() {
    let (_h, hub) = ingested_hub();
    let p = project(1, Some("personal"));
    let r = Retrieval::new(hub, p.path());
    let res = r
        .search(&AssetSearchRequest {
            query: "ticker".into(),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(res.mode, LicenseMode::Personal);
    assert_eq!(res.hits.len(), 1);
}

#[test]
fn the_budget_gate_limits_cards_and_full_source_per_decision() {
    let (_h, hub) = ingested_hub();
    let p = project(3, Some("personal"));
    let r = Retrieval::new(hub, p.path());
    let card = |id: &str| r.card(&AssetIdRequest { id: id.into() });
    card("pack-mit/glow-title").unwrap();
    card("pack-mit/glow-title").unwrap(); // re-reading the same card is not a new one
    card("pack-mit/slide-in").unwrap();
    let third = card("pack-restricted/news-ticker").unwrap_err();
    assert_eq!(third.code, ErrorCode::Budget);

    let src = |reason: &str, id: &str| {
        r.source(&AssetSourceRequest {
            id: id.into(),
            reason: reason.into(),
        })
    };
    assert_eq!(
        src("why", "pack-mit/slide-in").unwrap_err().code,
        ErrorCode::Request
    );
    let full = src("need the easing curve exactly", "pack-mit/slide-in").unwrap();
    assert!(full.text.contains("Easing.out"));
    assert!(full.budget.used > 0);
    assert_eq!(
        src("need the glow shadow exactly", "pack-mit/glow-title")
            .unwrap_err()
            .code,
        ErrorCode::Budget
    );

    // A revision does not reset the gate; only handoff, which clears the ledger, does.
    std::fs::write(
        p.path().join(".clipkit/project.json"),
        serde_json::json!({ "rev": 4, "brief": {} }).to_string(),
    )
    .unwrap();
    let r2 = Retrieval::new(r.hub.clone(), p.path());
    let kept = r2
        .card(&AssetIdRequest {
            id: "pack-mit/glow-title".into(),
        })
        .unwrap();
    assert_eq!(kept.budget.cards, 2);
    std::fs::remove_file(p.path().join(".clipkit").join(LEDGER_FILE)).unwrap();
    let again = r2
        .card(&AssetIdRequest {
            id: "pack-mit/glow-title".into(),
        })
        .unwrap();
    assert_eq!(again.budget.cards, 1);
}

#[test]
fn skeleton_and_slice_stay_small_and_unknown_assets_are_hidden() {
    let (_h, hub) = ingested_hub();
    let p = project(1, None);
    let r = Retrieval::new(hub, p.path());
    let sk = r
        .skeleton(&AssetIdRequest {
            id: "pack-mit/glow-title".into(),
        })
        .unwrap();
    assert!(
        sk.text.contains("props GlowTitleProps:\n  text: string"),
        "{}",
        sk.text
    );
    assert!(sk.text.contains("export const GlowTitle (lines"));
    assert!(sk.text.len() <= caps::SKELETON);
    let sl = r
        .slice(&AssetSliceRequest {
            id: "pack-mit/slide-in".into(),
            symbol: "SlideIn".into(),
        })
        .unwrap();
    assert!(sl.text.starts_with("export function SlideIn"));
    let none = r
        .card(&AssetIdRequest {
            id: "pack-injection/sparkle".into(),
        })
        .unwrap_err();
    assert_eq!(none.code, ErrorCode::NotFound);
}

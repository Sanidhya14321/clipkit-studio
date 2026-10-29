use super::*;
use crate::sources::*;

pub(crate) fn hub_with(entries: serde_json::Value) -> (tempfile::TempDir, Hub) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(SOURCES_FILE), entries.to_string()).unwrap();
    let hub = Hub::new(dir.path());
    (dir, hub)
}

#[test]
fn the_real_registry_loads_and_nothing_is_usable_yet() {
    let hub = Hub::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    let sources = hub.sources().unwrap();
    assert!(sources.len() >= 60);
    // Unknown fields (wave, phase, packages, ...) survive a load.
    assert!(sources.iter().any(|s| s.extra.contains_key("wave")));
    let gaps = sources_gaps(&hub, &SourcesGapsRequest {}).unwrap();
    assert_eq!(gaps.sources, sources.len());
}

#[test]
fn add_records_an_unverified_disabled_source_and_list_shows_it() {
    let (_d, hub) = hub_with(serde_json::json!([]));
    let r = sources_add(
        &hub,
        &SourcesAddRequest {
            id: "my-pack".into(),
            name: String::new(),
            kind: "remotion-components".into(),
            repo: Some("me/my-pack".into()),
            license: "MIT".into(),
        },
    )
    .unwrap();
    assert_eq!(
        (r.status, r.enabled, r.license_class),
        (SourceStatus::Unverified, false, LicenseClass::Unknown)
    );
    let list = sources_list(&hub, &SourcesListRequest::default()).unwrap();
    assert_eq!(
        list.lines,
        ["my-pack unverified disabled unknown remotion-components me/my-pack"]
    );
    let s = hub.source("my-pack").unwrap();
    assert_eq!(s.license, "Unverified (claimed: MIT)");

    let dup = SourcesAddRequest {
        id: "my-pack".into(),
        kind: "x".into(),
        ..Default::default()
    };
    assert_eq!(
        sources_add(&hub, &dup).unwrap_err().code,
        ck_core::ErrorCode::Request
    );
    for (id, repo) in [
        ("Bad Id", None),
        ("ok", Some("../etc")),
        ("ok", Some("a/b/c")),
    ] {
        let req = SourcesAddRequest {
            id: id.into(),
            kind: "x".into(),
            repo: repo.map(str::to_owned),
            ..Default::default()
        };
        assert_eq!(
            sources_add(&hub, &req).unwrap_err().code,
            ck_core::ErrorCode::Request
        );
    }
}

#[test]
fn gaps_count_only_usable_sources() {
    let (_d, hub) = hub_with(serde_json::json!([
        { "id": "a", "licenseClass": "permissive", "status": "vetted", "enabled": true },
        { "id": "b", "licenseClass": "unknown", "status": "unverified", "repo": null }
    ]));
    std::fs::create_dir_all(hub.assets()).unwrap();
    let line = |id: &str, src: &str| {
        serde_json::json!({
        "id": id, "name": id, "source": src, "kind": "remotion-clip", "runtime": "direct",
        "licenseClass": "permissive", "grade": "A", "aspects": [], "tags": [],
        "collections": [], "useWhen": "x" })
        .to_string()
    };
    std::fs::write(
        hub.assets().join("catalog.jsonl"),
        format!("{}\n{}\n", line("a/one", "a"), line("b/two", "b")),
    )
    .unwrap();
    let g = sources_gaps(&hub, &SourcesGapsRequest {}).unwrap();
    assert_eq!((g.usable, g.unverified), (1, 1));
    assert_eq!(g.without_repo, ["b"]);
    assert_eq!(g.cards_by_kind, [(crate::card::AssetKind::RemotionClip, 1)]);
    assert!(
        !g.kinds_missing
            .contains(&crate::card::AssetKind::RemotionClip)
    );
}

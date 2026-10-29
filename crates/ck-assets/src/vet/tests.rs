use super::*;
use crate::hub::tests::hub_with;

fn fixture(name: &str) -> String {
    format!(
        "{}/../../fixtures/sources/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn three_sources() -> (tempfile::TempDir, Hub) {
    hub_with(serde_json::json!([
        { "id": "pack-mit", "licenseClass": "unknown", "status": "unverified", "repo": null },
        { "id": "pack-restricted", "licenseClass": "unknown", "status": "unverified" },
        { "id": "pack-injection", "licenseClass": "unknown", "status": "unverified" }
    ]))
}

fn vet_local(hub: &Hub, id: &str) -> VetResult {
    vet(
        hub,
        &VetRequest {
            id: id.into(),
            from: Some(fixture(id)),
        },
    )
    .unwrap()
}

#[test]
fn a_clean_permissive_source_is_vetted_enabled_and_pinned() {
    let (_d, hub) = three_sources();
    let r = vet_local(&hub, "pack-mit");
    assert_eq!((r.status, r.enabled), (SourceStatus::Vetted, true));
    assert_eq!(
        (r.license_class, r.spdx.as_str()),
        (LicenseClass::Permissive, "MIT")
    );
    assert!(r.findings.is_empty(), "{:?}", r.findings);
    assert_eq!(r.files, 4);
    let lock = hub.lock().unwrap();
    assert_eq!(lock["pack-mit"].commit, r.commit);
    assert_eq!(lock["pack-mit"].license_hash.len(), 64);
    assert_eq!(hub.source("pack-mit").unwrap().license, "MIT");
    // Quarantined files are read-only; vetting again replaces them.
    let q = hub.quarantine().join("pack-mit/src/GlowTitle.tsx");
    assert!(std::fs::metadata(&q).unwrap().permissions().readonly());
    assert_eq!(vet_local(&hub, "pack-mit").commit, r.commit);
}

#[test]
fn custom_non_commercial_terms_vet_as_commercial_restricted() {
    let (_d, hub) = three_sources();
    let r = vet_local(&hub, "pack-restricted");
    assert_eq!(r.status, SourceStatus::Vetted);
    assert_eq!(
        (r.license_class, r.spdx.as_str()),
        (LicenseClass::CommercialRestricted, "")
    );
    assert_eq!(
        hub.source("pack-restricted").unwrap().license,
        "custom terms (see LICENSE)"
    );
}

#[test]
fn the_hostile_pack_is_rejected_and_its_text_is_not_followed() {
    let (_d, hub) = three_sources();
    let r = vet_local(&hub, "pack-injection");
    assert_eq!((r.status, r.enabled), (SourceStatus::Rejected, false));
    let rules: Vec<(&str, Severity)> = r
        .findings
        .iter()
        .map(|f| (f.rule.as_str(), f.severity))
        .collect();
    for rule in [
        "binary-blob",
        "install-script",
        "non-registry-dependency",
        "download-lure",
        "dynamic-code",
    ] {
        assert!(
            rules.contains(&(rule, Severity::Blocker)),
            "{rule}: {rules:?}"
        );
    }
    assert!(rules.contains(&("prompt-injection", Severity::Warning)));
    assert!(rules.contains(&("nondeterministic", Severity::Warning)));
    // It asked to be marked permissive and enabled; it is not.
    let s = hub.source("pack-injection").unwrap();
    assert_eq!((s.license_class, s.enabled), (LicenseClass::Unknown, false));
}

#[test]
fn unknown_sources_and_missing_repos_are_refused() {
    let (_d, hub) = three_sources();
    let missing = vet(
        &hub,
        &VetRequest {
            id: "nope".into(),
            from: None,
        },
    )
    .unwrap_err();
    assert_eq!(missing.code, ErrorCode::NotFound);
    let no_repo = vet(
        &hub,
        &VetRequest {
            id: "pack-mit".into(),
            from: None,
        },
    )
    .unwrap_err();
    assert_eq!(no_repo.code, ErrorCode::Request);
}

#[test]
fn the_date_is_a_calendar_date() {
    let d = today();
    assert_eq!(d.len(), 10);
    assert!(d.starts_with("20"));
}

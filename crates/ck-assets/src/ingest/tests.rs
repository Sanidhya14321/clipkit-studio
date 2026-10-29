use super::*;
use crate::hub::tests::hub_with;
use crate::vet::{VetRequest, vet};

fn fixture(name: &str) -> String {
    format!(
        "{}/../../fixtures/sources/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
}

pub(crate) fn ingested_hub() -> (tempfile::TempDir, Hub) {
    let (d, hub) = hub_with(serde_json::json!([
        { "id": "pack-mit", "licenseClass": "unknown", "status": "unverified" },
        { "id": "pack-restricted", "licenseClass": "unknown", "status": "unverified" },
        { "id": "pack-injection", "licenseClass": "unknown", "status": "unverified" }
    ]));
    for id in ["pack-mit", "pack-restricted", "pack-injection"] {
        vet(
            &hub,
            &VetRequest {
                id: id.into(),
                from: Some(fixture(id)),
            },
        )
        .unwrap();
    }
    for id in ["pack-mit", "pack-restricted"] {
        ingest(&hub, &IngestRequest { id: id.into() }).unwrap();
    }
    (d, hub)
}

#[test]
fn a_vetted_source_becomes_cards_ast_maps_and_catalog_lines() {
    let (_d, hub) = ingested_hub();
    let r = ingest(
        &hub,
        &IngestRequest {
            id: "pack-mit".into(),
        },
    )
    .unwrap();
    assert_eq!(r.assets, ["pack-mit/glow-title", "pack-mit/slide-in"]);
    assert_eq!(r.catalog, 3, "two MIT cards plus the restricted one");
    let dir = hub.assets().join("pack-mit/glow-title");
    let card = Card::parse(
        &dir.join("asset.md"),
        &std::fs::read_to_string(dir.join("asset.md")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        (card.kind, card.runtime, card.determinism, card.grade),
        (
            AssetKind::RemotionClip,
            Runtime::Direct,
            Determinism::Native,
            Grade::A
        )
    );
    assert_eq!((card.license.as_str(), card.spdx.as_str()), ("MIT", "MIT"));
    assert!(card.section("Inputs").contains("- color?: string"));
    assert!(card.deps.contains(&"remotion".to_owned()));
    assert!(dir.join("ast.json").is_file());
    assert!(
        std::fs::metadata(dir.join("src/GlowTitle.tsx"))
            .unwrap()
            .permissions()
            .readonly()
    );
    assert!(hub.assets().join("pack-mit/SOURCE.md").is_file());
    let index = std::fs::read_to_string(hub.assets().join("pack-mit/INDEX.md")).unwrap();
    assert_eq!(index.lines().filter(|l| l.starts_with("- ")).count(), 2);
}

#[test]
fn owner_notes_survive_reingestion() {
    let (_d, hub) = ingested_hub();
    let path = hub.assets().join("pack-mit/slide-in/asset.md");
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(
        &path,
        text.replace("## Owner notes\n", "## Owner notes\n\nGreat for lists."),
    )
    .unwrap();
    ingest(
        &hub,
        &IngestRequest {
            id: "pack-mit".into(),
        },
    )
    .unwrap();
    let card = Card::parse(&path, &std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(card.section("Owner notes"), "Great for lists.");
}

#[test]
fn rejected_and_unvetted_sources_are_not_ingested() {
    let (_d, hub) = ingested_hub();
    let err = ingest(
        &hub,
        &IngestRequest {
            id: "pack-injection".into(),
        },
    )
    .unwrap_err();
    assert_eq!(err.code, ErrorCode::Request);
    assert!(!hub.assets().join("pack-injection").exists());
    let catalog = std::fs::read_to_string(hub.assets().join("catalog.jsonl")).unwrap();
    assert!(!catalog.contains("pack-injection"));
}

#[test]
fn names_turn_kebab() {
    assert_eq!(kebab("GlowTitle"), "glow-title");
    assert_eq!(kebab("A"), "a");
}

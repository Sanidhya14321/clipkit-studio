use super::*;
use crate::ingest::tests::ingested_hub;

#[test]
fn colour_literals_are_found_once_each() {
    let src = r##"const a = "#FFF"; const b = '#00ff00'; const c = "#fff"; const d = "#12";"##;
    assert_eq!(colour_literals(src), ["#fff", "#00ff00"]);
}

#[test]
fn a_derived_copy_has_tokens_instrumentation_schema_and_provenance() {
    let (_h, hub) = ingested_hub();
    let project = tempfile::tempdir().unwrap();
    let d = derive(
        &hub,
        project.path(),
        "pack-mit/glow-title",
        "glow",
        LicenseMode::CommercialSafe,
    )
    .unwrap();
    assert_eq!(d.tokens, [("color1".to_owned(), "#ffffff".to_owned())]);
    let src = std::fs::read_to_string(project.path().join(".clipkit/derived/glow/GlowTitle.tsx"))
        .unwrap();
    assert!(src.contains("color = ckTokens.color1"), "{src}");
    assert!(src.contains("export const ckTokens = {\n  color1: \"#ffffff\",\n};"));
    assert!(
        src.contains("<AbsoluteFill data-ck-layer={ckLayerId}"),
        "{src}"
    );
    // The rewritten file still parses.
    assert_eq!(extract("GlowTitle.tsx", &src).errors, 0);
    let schema: Value = serde_json::from_str(
        &std::fs::read_to_string(
            project
                .path()
                .join(".clipkit/derived/glow/props.schema.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(schema["required"], json!(["text"]));
    assert_eq!(schema["properties"]["glow"]["type"], "number");
    assert_eq!(d.provenance.source_asset, "pack-mit/glow-title");
    assert_eq!(d.provenance.hash.len(), 64);
    assert_eq!(d.provenance.license, "MIT");
    assert!(
        d.provenance
            .adaptations
            .iter()
            .any(|a| a.starts_with("instrumented (1"))
    );
}

#[test]
fn the_license_mode_and_bad_names_refuse_a_derive() {
    let (_h, hub) = ingested_hub();
    let project = tempfile::tempdir().unwrap();
    let blocked = derive(
        &hub,
        project.path(),
        "pack-restricted/news-ticker",
        "ticker",
        LicenseMode::CommercialSafe,
    )
    .unwrap_err();
    assert_eq!(blocked.code, ErrorCode::Request);
    derive(
        &hub,
        project.path(),
        "pack-restricted/news-ticker",
        "ticker",
        LicenseMode::Personal,
    )
    .unwrap();
    let bad = derive(
        &hub,
        project.path(),
        "pack-mit/slide-in",
        "../x",
        LicenseMode::Any,
    )
    .unwrap_err();
    assert_eq!(bad.code, ErrorCode::Request);
    let missing = derive(
        &hub,
        project.path(),
        "pack-injection/sparkle",
        "s",
        LicenseMode::Any,
    )
    .unwrap_err();
    assert_eq!(missing.code, ErrorCode::NotFound);
}

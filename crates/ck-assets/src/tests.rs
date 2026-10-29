use std::path::{Path, PathBuf};

use ck_core::ErrorCode;

use super::*;

fn assets_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets")
}

#[test]
fn starter_taste_library_has_twelve_valid_files() {
    let files = load_taste(&assets_dir().join("taste")).unwrap();
    let ids: Vec<&str> = files.iter().map(|f| f.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "audio-mixing",
            "captions",
            "color",
            "hooks",
            "layout",
            "motion",
            "pacing",
            "platform-specs",
            "sound-design",
            "transitions",
            "typography",
            "video-types",
        ]
    );
}

#[test]
fn starter_looks_are_the_four_of_spec_7_5() {
    let looks = load_looks(&assets_dir().join("looks")).unwrap();
    let ids: Vec<&str> = looks.iter().map(|l| l.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "bold-social",
            "calm-documentary",
            "cinematic",
            "clean-product"
        ]
    );
}

#[test]
fn taste_parse_refusals_carry_e_asset() {
    let p = Path::new("taste/pacing.md");
    let cases = [
        "no front matter",
        "---\nid: pacing\ntitle: T\ntags: [a]\nbody 1",
        "---\nid: other\ntitle: T\ntags: [a]\n---\nbody 1",
        "---\nid: pacing\ntitle: T\ntags: a\n---\nbody 1",
        "---\nid: pacing\ntitle: T\ntags: [a]\nextra: x\n---\nbody 1",
        "---\nid: pacing\ntitle: T\ntags: [a]\n---\nno numbers here",
    ];
    for text in cases {
        let err = TasteFile::parse(p, text).unwrap_err();
        assert_eq!(err.code, ErrorCode::Asset, "{text}");
    }
    let big = format!(
        "---\nid: pacing\ntitle: T\ntags: [a]\n---\n1{}",
        "x".repeat(2400)
    );
    let err = TasteFile::parse(p, &big).unwrap_err();
    assert!(err.message.contains("byte cap"), "{}", err.message);
    let ok = TasteFile::parse(p, "---\nid: pacing\ntitle: T\ntags: [a, b]\n---\nbody 1\n").unwrap();
    assert_eq!(ok.tags, ["a", "b"]);
    assert_eq!(ok.body, "body 1");
}

fn good_look() -> serde_json::Value {
    let text = std::fs::read_to_string(assets_dir().join("looks/clean-product.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn parse_look(v: &serde_json::Value) -> Result<Look, ck_core::CkError> {
    Look::parse(Path::new("looks/clean-product.json"), &v.to_string())
}

#[test]
fn look_refusals_carry_e_asset() {
    assert!(parse_look(&good_look()).is_ok());
    let mutations: [(&str, serde_json::Value); 6] = [
        ("/colors/text", "#F0F0F0".into()),
        ("/colors/accent", "blue".into()),
        ("/colors/accentText", "#3050E0".into()),
        ("/id", "bold-social".into()),
        ("/motion/durationScale", 5.0.into()),
        ("/fonts/display", " ".into()),
    ];
    for (ptr, value) in mutations {
        let mut v = good_look();
        *v.pointer_mut(ptr).unwrap() = value;
        let err = parse_look(&v).unwrap_err();
        assert_eq!(err.code, ErrorCode::Asset, "{ptr}");
    }
    let mut v = good_look();
    v["fonts"]["third"] = "Comic Sans".into();
    assert_eq!(parse_look(&v).unwrap_err().code, ErrorCode::Asset);
}

#[test]
fn contrast_matches_wcag_reference_values() {
    let black = color::parse_hex("#000000").unwrap();
    let white = color::parse_hex("#FFFFFF").unwrap();
    assert!((color::contrast(black, white) - 21.0).abs() < 1e-9);
    assert!((color::contrast(white, white) - 1.0).abs() < 1e-9);
    // #777777 on white is the well-known 4.48 just under AA.
    let grey = color::parse_hex("#777777").unwrap();
    assert!((color::contrast(grey, white) - 4.48).abs() < 0.01);
    assert!(color::parse_hex("#12345").is_none());
    assert!(color::parse_hex("123456").is_none());
}

#[test]
fn missing_dir_is_e_storage() {
    let dir = tempfile::tempdir().unwrap();
    let err = load_looks(&dir.path().join("nope")).unwrap_err();
    assert_eq!(err.code, ErrorCode::Storage);
}

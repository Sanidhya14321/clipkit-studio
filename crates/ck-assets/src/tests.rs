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

#[test]
fn built_in_library_matches_the_assets_folder() {
    assert_eq!(
        library::taste().unwrap(),
        load_taste(&assets_dir().join("taste")).unwrap()
    );
    assert_eq!(
        library::looks().unwrap(),
        load_looks(&assets_dir().join("looks")).unwrap()
    );
}

#[test]
fn taste_search_ranks_by_head_then_body_and_stays_under_its_cap() {
    use library::{TasteSearchRequest, caps, taste_search};
    let all = taste_search(&TasteSearchRequest::default()).unwrap();
    assert_eq!(all.total, 12);
    assert_eq!(all.hits.len(), library::DEFAULT_HITS);
    let found = taste_search(&TasteSearchRequest {
        query: "Pacing".into(),
        limit: Some(100),
    })
    .unwrap();
    assert_eq!(found.hits[0].id, "pacing");
    assert_eq!(found.hits[0].score, 3);
    assert!(found.hits.len() <= library::MAX_HITS);
    let none = taste_search(&TasteSearchRequest {
        query: "zzqx".into(),
        limit: None,
    })
    .unwrap();
    assert_eq!(none.total, 0);
    let widest = taste_search(&TasteSearchRequest {
        query: String::new(),
        limit: Some(library::MAX_HITS),
    })
    .unwrap();
    let len = serde_json::to_string(&widest).unwrap().len();
    assert!(len <= caps::TASTE_SEARCH, "{len}");
}

#[test]
fn reads_stay_under_their_caps_and_unknown_ids_are_not_found() {
    use library::{IdRequest, LookListRequest, caps};
    for f in library::taste().unwrap() {
        let got = library::taste_read(&IdRequest { id: f.id.clone() }).unwrap();
        let len = serde_json::to_string(&got).unwrap().len();
        assert!(len <= caps::TASTE_READ, "{} {len}", f.id);
    }
    for l in library::looks().unwrap() {
        let got = library::look_get(&IdRequest { id: l.id.clone() }).unwrap();
        let len = serde_json::to_string(&got).unwrap().len();
        assert!(len <= caps::LOOK_GET, "{} {len}", l.id);
    }
    let list = library::look_list(&LookListRequest {}).unwrap();
    assert_eq!(list.looks.len(), 4);
    assert!(serde_json::to_string(&list).unwrap().len() <= caps::LOOK_LIST);
    let id = IdRequest { id: "nope".into() };
    assert_eq!(
        library::taste_read(&id).unwrap_err().code,
        ErrorCode::NotFound
    );
    assert_eq!(
        library::look_get(&id).unwrap_err().code,
        ErrorCode::NotFound
    );
}

#[test]
fn assets_index_is_short_and_names_every_built_in_file() {
    let index = std::fs::read_to_string(assets_dir().join("INDEX.md")).unwrap();
    assert!(index.lines().count() < 60);
    for (id, _) in library::TASTE_FILES.iter().chain(library::LOOK_FILES) {
        assert!(index.contains(id), "INDEX.md misses {id}");
    }
}

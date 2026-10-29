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
fn recipe_search_lists_slots_ranks_and_stays_under_its_cap() {
    use library::{IdRequest, RecipeSearchRequest, caps};
    let all = library::recipe_search(&RecipeSearchRequest {
        query: String::new(),
        limit: Some(library::MAX_HITS),
    })
    .unwrap();
    assert_eq!((all.total, all.hits.len()), (10, 10));
    let len = serde_json::to_string(&all).unwrap().len();
    assert!(len <= caps::RECIPE_SEARCH, "{len}");

    let hook = library::recipe_search(&RecipeSearchRequest {
        query: "hook".into(),
        limit: None,
    })
    .unwrap();
    assert_eq!(hook.hits[0].id, "hook-title");
    assert_eq!(
        hook.hits[0].slots,
        ["headline*:text<=48", "kicker:text<=24"]
    );
    assert_eq!(hook.hits[0].duration, 2.5);

    for (id, _) in library::RECIPE_FILES {
        let card = library::recipe_card(&IdRequest { id: (*id).into() }).unwrap();
        let len = serde_json::to_string(&card).unwrap().len();
        assert!(len <= caps::RECIPE_CARD, "{id} {len}");
    }
    let id = IdRequest { id: "nope".into() };
    assert_eq!(
        library::recipe_card(&id).unwrap_err().code,
        ErrorCode::NotFound
    );
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
    for (id, _) in library::TASTE_FILES
        .iter()
        .chain(library::LOOK_FILES)
        .chain(library::RECIPE_FILES)
    {
        assert!(index.contains(id), "INDEX.md misses {id}");
    }
}

mod recipes {
    use ck_core::model::{ItemBody, TrackKind};
    use serde_json::{Map, Value, json};

    use super::*;
    use crate::recipe::{ExpandInput, MAX_RECIPE_BYTES, fnv1a, token_value};

    fn recipe(id: &str) -> (Recipe, &'static str) {
        let (_, text) = library::RECIPE_FILES
            .iter()
            .find(|(i, _)| *i == id)
            .unwrap();
        let r = library::recipes()
            .unwrap()
            .into_iter()
            .find(|r| r.id == id)
            .unwrap();
        (r, text)
    }

    fn look() -> Look {
        library::looks()
            .unwrap()
            .into_iter()
            .find(|l| l.id == "clean-product")
            .unwrap()
    }

    fn slots(v: Value) -> Map<String, Value> {
        v.as_object().unwrap().clone()
    }

    fn input<'a>(slots: &'a Map<String, Value>, sources: &'a [String]) -> ExpandInput<'a> {
        ExpandInput {
            prefix: "s1",
            at: 2.0,
            duration: None,
            slots,
            sources,
        }
    }

    #[test]
    fn ten_recipes_of_spec_7_5_load_and_match_the_folder() {
        let all = library::recipes().unwrap();
        assert_eq!(all.len(), 10);
        assert_eq!(all, load_recipes(&assets_dir().join("recipes")).unwrap());
        for (id, text) in library::RECIPE_FILES {
            assert!(text.len() <= MAX_RECIPE_BYTES, "{id} too large");
        }
    }

    #[test]
    fn every_token_a_recipe_uses_resolves_in_every_look() {
        for r in library::recipes().unwrap() {
            for l in &r.layers {
                for v in l.props.values() {
                    let Some(t) = v.as_str().and_then(|s| s.strip_prefix('$')) else {
                        continue;
                    };
                    for look in library::looks().unwrap() {
                        assert!(
                            token_value(&look, t).is_some(),
                            "{} ${t} in {}",
                            r.id,
                            look.id
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn expand_builds_named_layers_with_times_tokens_and_provenance() {
        let (r, text) = recipe("hook-title");
        let s = slots(json!({ "headline": "Ship video in minutes" }));
        let out = r.expand(&look(), text, &input(&s, &[])).unwrap();
        assert_eq!(out.layers.len(), 2);
        let bg = &out.layers[0];
        assert_eq!(bg.track, TrackKind::Video);
        assert_eq!(bg.item.id, "s1-backdrop");
        assert_eq!(bg.item.asset_ref.as_deref(), Some("recipe:hook-title"));
        assert_eq!(bg.item.brief_deps, vec!["look", "aspect"]);
        let head = &out.layers[1];
        assert_eq!(head.track, TrackKind::Overlay);
        let ItemBody::Clip {
            clip_id,
            at,
            dur,
            props,
            ..
        } = &head.item.body
        else {
            panic!("not a clip")
        };
        assert_eq!(clip_id.as_deref(), Some("kinetic-text"));
        assert_eq!((*at, *dur), (2.1, 2.4));
        assert_eq!(props["text"], "Ship video in minutes");
        // Tokens stay references so a re-theme costs no agent tokens.
        assert_eq!(props["color"], "$colors.text");
        // The optional kicker was left out: its prop is dropped, never a placeholder.
        assert!(props.get("kicker").is_none());
        assert_eq!(out.provenance.source_asset, "recipe:hook-title");
        assert_eq!(out.provenance.hash, fnv1a(text));
        assert_eq!(out.provenance.adaptations, vec!["headline"]);
    }

    #[test]
    fn expand_checks_slots_and_duration() {
        let (r, text) = recipe("kinetic-list");
        let look = look();
        let bad = [
            json!({ "title": "Why" }),
            json!({ "title": "Why", "items": ["one"] }),
            json!({ "title": "Why", "items": ["one", ""] }),
            json!({ "title": "Why", "items": ["a", "b"], "extra": 1 }),
            json!({ "title": "x".repeat(41), "items": ["a", "b"] }),
            json!({ "title": 3, "items": ["a", "b"] }),
        ];
        for v in bad {
            let s = slots(v.clone());
            let err = r.expand(&look, text, &input(&s, &[])).unwrap_err();
            assert_eq!(err.code, ErrorCode::Request, "{v}");
        }
        let s = slots(json!({ "title": "Why", "items": ["a", "b"] }));
        let mut i = input(&s, &[]);
        i.duration = Some(20.0);
        assert_eq!(
            r.expand(&look, text, &i).unwrap_err().code,
            ErrorCode::Request
        );
        i.duration = Some(4.0);
        i.at = -1.0;
        assert_eq!(
            r.expand(&look, text, &i).unwrap_err().code,
            ErrorCode::Request
        );
    }

    #[test]
    fn source_slots_need_a_project_source() {
        let (r, text) = recipe("product-reveal");
        let s = slots(json!({ "media": "src-1", "name": "ClipKit" }));
        let err = r.expand(&look(), text, &input(&s, &[])).unwrap_err();
        assert_eq!(err.code, ErrorCode::Request);
        let sources = ["src-1".to_string()];
        let out = r.expand(&look(), text, &input(&s, &sources)).unwrap();
        let ItemBody::Clip { props, .. } = &out.layers[1].item.body else {
            panic!("not a clip")
        };
        assert_eq!(props["source"], "src-1");
    }

    #[test]
    fn recipe_refusals_carry_e_asset() {
        let (_, text) = recipe("hook-title");
        let good: Value = serde_json::from_str(text).unwrap();
        let path = Path::new("assets/recipes/hook-title.json");
        let cases: [(&str, Value); 8] = [
            ("/id", json!("other")),
            ("/duration/min", json!(9.0)),
            ("/slots/0/required", json!(false)),
            ("/slots/1/name", json!("headline")),
            ("/layers/0/clip", json!("confetti")),
            ("/layers/0/end", json!(1.5)),
            ("/layers/0/briefDeps", json!(["mood"])),
            ("/layers/1/props/color", json!("$colors.neon")),
        ];
        for (ptr, v) in cases {
            let mut bad = good.clone();
            *bad.pointer_mut(ptr).unwrap() = v;
            let err = Recipe::parse(path, &bad.to_string()).unwrap_err();
            assert_eq!(err.code, ErrorCode::Asset, "{ptr}");
        }
        let mut unused = good.clone();
        unused["layers"][1]["props"]
            .as_object_mut()
            .unwrap()
            .remove("kicker");
        assert_eq!(
            Recipe::parse(path, &unused.to_string()).unwrap_err().code,
            ErrorCode::Asset
        );
        let big = format!("{text}{}", " ".repeat(MAX_RECIPE_BYTES));
        assert_eq!(
            Recipe::parse(path, &big).unwrap_err().code,
            ErrorCode::Asset
        );
    }
}

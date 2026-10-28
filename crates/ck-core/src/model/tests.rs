use proptest::prelude::*;

use super::*;
use crate::ErrorCode;

/// A small valid project used across tests.
pub(crate) fn sample() -> Project {
    let mut p = Project::new("p1", "Sample", Mode::Hybrid);
    p.sources.push(Source {
        id: "s1".into(),
        kind: SourceKind::Video,
        path: "media/clip.mp4".into(),
        proxy_path: None,
        hash: "abc".into(),
        duration_sec: Some(12.0),
        transcript_path: None,
    });
    p.voice_slots.push(VoiceSlot {
        id: "v1".into(),
        text: "Hello".into(),
        mode: VoiceMode::Ai,
        voice_ref: None,
        takes: vec![],
        active_take: None,
        target_window: None,
        status: VoiceSlotStatus::Empty,
    });
    let item = |id: &str, body| Item {
        id: id.into(),
        name: id.into(),
        purpose: String::new(),
        pass: None,
        asset_ref: None,
        brief_deps: vec![],
        rev: 0,
        body,
    };
    p.tracks.push(Track {
        id: "t1".into(),
        kind: TrackKind::Video,
        items: vec![item(
            "a",
            ItemBody::Footage {
                src: "s1".into(),
                src_in: 1.0,
                src_out: 5.0,
                at: 0.0,
                speed: 1.0,
                volume: 1.0,
            },
        )],
    });
    p.tracks.push(Track {
        id: "t2".into(),
        kind: TrackKind::Overlay,
        items: vec![
            item(
                "b",
                ItemBody::Clip {
                    clip_id: Some("title".into()),
                    derived_id: None,
                    at: 0.5,
                    dur: 2.0,
                    props: serde_json::json!({"text": "Hi"}),
                },
            ),
            item(
                "c",
                ItemBody::Voice {
                    slot_id: "v1".into(),
                    at: 1.0,
                },
            ),
        ],
    });
    p
}

#[test]
fn sample_is_valid_and_round_trips() {
    let p = sample();
    p.validate().unwrap();
    let json = serde_json::to_string(&p).unwrap();
    assert_eq!(serde_json::from_str::<Project>(&json).unwrap(), p);
}

#[test]
fn item_body_is_tagged_and_camel_case() {
    let v = serde_json::to_value(sample()).unwrap();
    let a = &v["tracks"][0]["items"][0];
    assert_eq!(a["type"], "footage");
    assert_eq!(a["srcIn"], 1.0);
    assert_eq!(v["schemaVersion"], SCHEMA_VERSION);
}

#[test]
fn json_schema_is_stable() {
    insta::assert_json_snapshot!(schemars::schema_for!(Project));
}

type Mutation = Box<dyn Fn(&mut Project)>;

#[test]
fn validation_rejects_bad_projects() {
    let cases: Vec<(&str, Mutation)> = vec![
        (
            "duplicate id",
            Box::new(|p| p.tracks[1].items[0].id = "a".into()),
        ),
        (
            "unknown source",
            Box::new(|p| {
                if let ItemBody::Footage { src, .. } = &mut p.tracks[0].items[0].body {
                    *src = "nope".into();
                }
            }),
        ),
        (
            "out before in",
            Box::new(|p| {
                if let ItemBody::Footage { src_out, .. } = &mut p.tracks[0].items[0].body {
                    *src_out = 0.5;
                }
            }),
        ),
        (
            "nan time",
            Box::new(|p| {
                if let ItemBody::Clip { at, .. } = &mut p.tracks[1].items[0].body {
                    *at = f64::NAN;
                }
            }),
        ),
        ("zero fps", Box::new(|p| p.settings.fps = 0)),
        (
            "unknown voice slot",
            Box::new(|p| {
                if let ItemBody::Voice { slot_id, .. } = &mut p.tracks[1].items[1].body {
                    *slot_id = "v9".into();
                }
            }),
        ),
    ];
    for (name, mutate) in cases {
        let mut p = sample();
        mutate(&mut p);
        let err = p.validate().expect_err(name);
        assert_eq!(err.code, ErrorCode::Schema, "{name}");
    }
}

proptest! {
    #[test]
    fn clip_items_round_trip(at in 0.0f64..1e6, dur in 1e-3f64..1e4, text in ".{0,20}") {
        let mut p = sample();
        p.tracks[1].items[0].body = ItemBody::Clip {
            clip_id: None, derived_id: Some("d1".into()), at, dur, props: serde_json::json!({ "text": text }),
        };
        p.validate().unwrap();
        let back: Project = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        prop_assert_eq!(back, p);
    }
}

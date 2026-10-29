//! Timeline math: seconds <-> frames, item spans and overlap detection.
//! Times are stored in seconds; frames exist only at render time (spec 3.2).

use crate::model::{Item, ItemBody, Project, TimeRange, Track};

/// Two times closer than this are treated as equal (touching items do not overlap).
pub const EPS: f64 = 1e-9;

/// Nearest frame for a time in seconds. Negative or non-finite times map to frame 0.
pub fn secs_to_frames(secs: f64, fps: u32) -> u64 {
    let f = (secs * f64::from(fps)).round();
    if f.is_finite() && f > 0.0 {
        f as u64
    } else {
        0
    }
}

/// Start time of a frame in seconds.
pub fn frames_to_secs(frames: u64, fps: u32) -> f64 {
    if fps == 0 {
        0.0
    } else {
        frames as f64 / f64::from(fps)
    }
}

/// Where an item sits on the timeline, or `None` when its length is not known yet
/// (a voice slot without an active take, or captions, which follow their words).
pub fn span(project: &Project, item: &Item) -> Option<TimeRange> {
    let (start, len) = match &item.body {
        ItemBody::Footage {
            src_in,
            src_out,
            at,
            speed,
            ..
        } => (*at, (src_out - src_in) / speed),
        ItemBody::Clip { at, dur, .. } => (*at, *dur),
        ItemBody::Voice { slot_id, at } => {
            let slot = project.voice_slots.iter().find(|s| &s.id == slot_id)?;
            let take_id = slot.active_take.as_ref()?;
            let take = slot.takes.iter().find(|t| &t.id == take_id)?;
            (*at, take.duration_sec)
        }
        ItemBody::Audio { at, ranges, .. } => (*at, ranges.iter().map(|r| r.end - r.start).sum()),
        ItemBody::Caption { .. } => return None,
    };
    (start.is_finite() && len.is_finite() && len >= 0.0).then_some(TimeRange {
        start,
        end: start + len,
    })
}

/// Every pair of items on the track whose spans overlap, as `(earlier id, later id)`.
/// Items without a known span are ignored. Touching items (end == start) do not overlap.
pub fn overlaps(project: &Project, track: &Track) -> Vec<(String, String)> {
    let mut spans: Vec<(TimeRange, &str)> = track
        .items
        .iter()
        .filter_map(|i| span(project, i).map(|s| (s, i.id.as_str())))
        .collect();
    spans.sort_by(|a, b| a.0.start.total_cmp(&b.0.start).then(a.1.cmp(b.1)));

    let mut out = Vec::new();
    for (i, (a, a_id)) in spans.iter().enumerate() {
        for (b, b_id) in &spans[i + 1..] {
            if b.start >= a.end - EPS {
                break;
            }
            if b.end > b.start + EPS && a.end > a.start + EPS {
                out.push(((*a_id).to_owned(), (*b_id).to_owned()));
            }
        }
    }
    out
}

/// End of the last item with a known span, in seconds (0 for an empty project).
pub fn duration(project: &Project) -> f64 {
    project
        .items()
        .filter_map(|i| span(project, i))
        .map(|s| s.end)
        .fold(0.0, f64::max)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::model::{Mode, TrackKind};

    fn clip(id: &str, at: f64, dur: f64) -> Item {
        Item {
            id: id.into(),
            name: String::new(),
            purpose: String::new(),
            pass: None,
            asset_ref: None,
            brief_deps: vec![],
            provenance: None,
            rev: 0,
            body: ItemBody::Clip {
                clip_id: None,
                derived_id: None,
                at,
                dur,
                props: serde_json::Value::Null,
            },
        }
    }

    fn track(items: Vec<Item>) -> Track {
        Track {
            id: "t".into(),
            kind: TrackKind::Overlay,
            items,
        }
    }

    fn project() -> Project {
        Project::new("p", "p", Mode::Generate)
    }

    #[test]
    fn frames_round_to_nearest() {
        assert_eq!(secs_to_frames(1.0, 30), 30);
        assert_eq!(secs_to_frames(1.0 / 30.0 * 0.49, 30), 0);
        assert_eq!(secs_to_frames(1.0 / 30.0 * 0.51, 30), 1);
        assert_eq!(secs_to_frames(-3.0, 30), 0);
        assert_eq!(secs_to_frames(f64::NAN, 30), 0);
        assert_eq!(frames_to_secs(45, 30), 1.5);
        assert_eq!(frames_to_secs(10, 0), 0.0);
    }

    #[test]
    fn footage_span_accounts_for_speed() {
        let mut p = crate::model::tests::sample();
        if let ItemBody::Footage { speed, .. } = &mut p.tracks[0].items[0].body {
            *speed = 2.0;
        }
        let s = span(&p, &p.tracks[0].items[0]).unwrap();
        assert_eq!((s.start, s.end), (0.0, 2.0)); // 4 s of source at 2x
    }

    #[test]
    fn voice_span_needs_an_active_take() {
        let mut p = crate::model::tests::sample();
        let item = p.tracks[1].items[1].clone();
        assert!(span(&p, &item).is_none());
        p.voice_slots[0].takes.push(crate::model::VoiceTake {
            id: "k1".into(),
            path: "v.wav".into(),
            duration_sec: 1.25,
            recorded: false,
            consented: false,
        });
        p.voice_slots[0].active_take = Some("k1".into());
        assert_eq!(span(&p, &item).unwrap().end, 2.25);
    }

    #[test]
    fn touching_items_do_not_overlap() {
        let t = track(vec![clip("a", 0.0, 1.0), clip("b", 1.0, 1.0)]);
        assert!(overlaps(&project(), &t).is_empty());
        let t = track(vec![
            clip("a", 0.0, 1.0),
            clip("b", 0.5, 1.0),
            clip("c", 0.9, 0.05),
        ]);
        assert_eq!(overlaps(&project(), &t).len(), 3);
    }

    proptest! {
        #[test]
        fn frames_round_trip(frames in 0u64..(1 << 40), fps in 1u32..240) {
            prop_assert_eq!(secs_to_frames(frames_to_secs(frames, fps), fps), frames);
        }

        #[test]
        fn back_to_back_items_never_overlap(durs in prop::collection::vec(1e-3f64..100.0, 1..30)) {
            let mut at = 0.0;
            let items: Vec<Item> = durs.iter().enumerate().map(|(i, d)| {
                let it = clip(&format!("i{i}"), at, *d);
                at += d;
                it
            }).collect();
            let p = project();
            prop_assert!(overlaps(&p, &track(items.clone())).is_empty());
            let full = Project { tracks: vec![track(items)], ..p };
            prop_assert!((duration(&full) - at).abs() < 1e-6);
        }

        #[test]
        fn overlap_set_ignores_item_order(
            spans in prop::collection::vec((0.0f64..50.0, 1e-3f64..10.0), 1..20),
            seed in any::<u64>(),
        ) {
            let items: Vec<Item> = spans.iter().enumerate().map(|(i, (a, d))| clip(&format!("i{i:02}"), *a, *d)).collect();
            let mut shuffled = items.clone();
            let n = shuffled.len();
            for i in 0..n { shuffled.swap(i, (seed as usize).wrapping_add(i * 7) % n); }
            let p = project();
            let norm = |v: Vec<(String, String)>| { let mut v: Vec<_> = v.into_iter().map(|(a, b)| if a < b { (a, b) } else { (b, a) }).collect(); v.sort(); v };
            let a = norm(overlaps(&p, &track(items.clone())));
            let b = norm(overlaps(&p, &track(shuffled)));
            prop_assert_eq!(&a, &b);
            // Brute force agrees.
            let mut brute = Vec::new();
            for i in 0..n { for j in i + 1..n {
                let (x, y) = (spans[i], spans[j]);
                if x.0 < y.0 + y.1 - EPS && y.0 < x.0 + x.1 - EPS { brute.push((format!("i{i:02}"), format!("i{j:02}"))); }
            }}
            prop_assert_eq!(a, norm(brute));
        }
    }
}

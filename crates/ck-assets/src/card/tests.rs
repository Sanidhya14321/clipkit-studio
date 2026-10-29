use std::path::Path;

use super::*;

pub(crate) fn sample() -> String {
    let mut s = String::from(
        "---\nid: pack-mit/glow-title\nname: Glow title\nsource: pack-mit\nsourceCommit: abc123\n\
         kind: remotion-clip\nruntime: direct\ndeterminism: native\nlicense: MIT\nspdx: MIT\n\
         licenseClass: permissive\ngrade: A\ndeps: [remotion]\ncost: 2\nduration: 3\n\
         aspects: [16:9, 9:16]\ntags: [title, glow]\ncollections: [starter]\nreviewed: false\n---\n",
    );
    for t in SECTIONS {
        let body = match *t {
            "Owner notes" => "",
            "Use when" => "- A hook title that should glow in.\n- Dark backgrounds.",
            _ => "Text.",
        };
        s.push_str(&format!("\n## {t}\n\n{body}\n"));
    }
    s
}

fn p() -> &'static Path {
    Path::new("asset.md")
}

#[test]
fn a_card_parses_renders_and_round_trips() {
    let c = Card::parse(p(), &sample()).unwrap();
    assert_eq!(c.id, "pack-mit/glow-title");
    assert_eq!(c.kind, AssetKind::RemotionClip);
    assert_eq!(c.aspects, ["16:9", "9:16"]);
    assert_eq!(c.duration, Some(3.0));
    let again = Card::parse(p(), &c.render(None)).unwrap();
    assert_eq!(again, c);
    let e = CatalogEntry::from(&c);
    assert_eq!(e.use_when, "A hook title that should glow in.");
}

#[test]
fn regeneration_keeps_the_owner_notes() {
    let c = Card::parse(p(), &sample()).unwrap();
    let text = c.render(Some("Looks best at 60 fps."));
    let back = Card::parse(p(), &text).unwrap();
    assert_eq!(back.section("Owner notes"), "Looks best at 60 fps.");
}

#[test]
fn bad_cards_are_asset_errors() {
    let s = sample();
    for bad in [
        s.replace("kind: remotion-clip", "kind: video"),
        s.replace("cost: 2", "cost: 9"),
        s.replace("reviewed: false\n", ""),
        s.replace("reviewed: false", "reviewed: false\nsecret: x"),
        s.replace("## Cannot do", "## Something else"),
        s.replace("id: pack-mit/glow-title", "id: other/glow-title"),
        s.replace("licenseClass: permissive", "licenseClass: attribution"),
        s.replace("Text.", ""),
        format!("{s}{}", "x".repeat(MAX_CARD_BYTES)),
    ] {
        assert_eq!(
            Card::parse(p(), &bad).unwrap_err().code,
            ck_core::ErrorCode::Asset
        );
    }
}

#[test]
fn license_modes_gate_classes() {
    use LicenseClass as L;
    let cs = LicenseMode::CommercialSafe;
    assert!(cs.allows(L::Permissive) && cs.allows(L::Attribution));
    for c in [
        L::Copyleft,
        L::CommercialRestricted,
        L::AccountRequired,
        L::Unknown,
        L::ReferenceOnly,
    ] {
        assert!(!cs.allows(c), "{c:?}");
    }
    assert!(LicenseMode::Personal.allows(L::CommercialRestricted));
    assert!(!LicenseMode::Any.allows(L::Unknown));
}

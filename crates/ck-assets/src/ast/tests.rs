use super::*;

fn fixture(p: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/../../fixtures/sources/{p}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

#[test]
fn a_remotion_component_maps_imports_props_primitives_and_outline() {
    let m = extract("GlowTitle.tsx", &fixture("pack-mit/src/GlowTitle.tsx"));
    assert_eq!(m.errors, 0);
    assert_eq!(m.imports[0].from, "remotion");
    assert_eq!(m.exports, ["GlowTitleProps", "GlowTitle"]);
    assert_eq!(
        m.primitives,
        [
            "useCurrentFrame",
            "useVideoConfig",
            "interpolate",
            "spring",
            "AbsoluteFill"
        ]
    );
    let props = &m.props[0];
    assert_eq!(props.name, "GlowTitleProps");
    let fields: Vec<(&str, &str, bool)> = props
        .fields
        .iter()
        .map(|f| (f.name.as_str(), f.ty.as_str(), f.optional))
        .collect();
    assert_eq!(
        fields,
        [
            ("text", "string", false),
            ("color", "string", true),
            ("glow", "number", true)
        ]
    );
    let comp = m.outline.iter().find(|s| s.name == "GlowTitle").unwrap();
    assert_eq!((comp.kind.as_str(), comp.exported), ("const", true));
    assert!(comp.start_line < comp.end_line);
    let iface = m
        .outline
        .iter()
        .find(|s| s.name == "GlowTitleProps")
        .unwrap();
    assert!(
        iface.doc.starts_with("A title that fades in"),
        "{}",
        iface.doc
    );
}

#[test]
fn type_alias_props_and_function_components_are_found() {
    let m = extract("SlideIn.tsx", &fixture("pack-mit/src/SlideIn.tsx"));
    assert_eq!(m.props[0].name, "SlideInProps");
    assert_eq!(m.props[0].fields.len(), 3);
    let f = m.outline.iter().find(|s| s.name == "SlideIn").unwrap();
    assert_eq!(f.kind, "function");
    assert!(m.primitives.contains(&"Easing".to_owned()));
    let body = slice(
        &fixture("pack-mit/src/SlideIn.tsx"),
        f.start_line,
        f.end_line,
    );
    assert!(body.starts_with("export function SlideIn"));
    assert!(body.trim_end().ends_with('}'));
}

#[test]
fn broken_source_still_maps_without_running_anything() {
    let m = extract(
        "Bad.tsx",
        "export const A = () => <div>;\nexport interface BProps { x: number }",
    );
    assert!(m.errors > 0);
}

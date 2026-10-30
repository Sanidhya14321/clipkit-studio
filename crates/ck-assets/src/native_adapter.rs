//! Explicit native technique adapters. Derived source files are never executed.
use ck_core::{
    Project,
    model::{Item, ItemBody},
};

/// How a native technique sits in a video.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeKind {
    Text,
    Overlay,
    Transition,
}

/// Cards with a clean-room native clip: asset id, adapter id, clip id, kind. These are the only
/// external cards a project can render.
pub const NATIVE: &[(&str, &str, &str, NativeKind)] = &[
    (
        "clipkit/kinetic-slam",
        "kinetic-slam-v1",
        "kinetic-slam",
        NativeKind::Text,
    ),
    (
        "rendercomp-free/glitch-text",
        "glitch-text-v1",
        "glitch-text",
        NativeKind::Text,
    ),
    (
        "rendercomp-free/typewriter",
        "typewriter-v1",
        "typewriter",
        NativeKind::Text,
    ),
    (
        "rendercomp-free/scramble-text",
        "scramble-text-v1",
        "scramble-text",
        NativeKind::Text,
    ),
    (
        "rendercomp-free/wave-text",
        "wave-text-v1",
        "wave-text",
        NativeKind::Text,
    ),
    (
        "rendercomp-free/gradient-text-sweep",
        "gradient-text-sweep-v1",
        "gradient-text-sweep",
        NativeKind::Text,
    ),
    (
        "rendercomp-free/bounce-in-headline",
        "bounce-in-headline-v1",
        "bounce-in-headline",
        NativeKind::Text,
    ),
    (
        "rendercomp-free/particle-snow",
        "particle-snow-v1",
        "particle-snow",
        NativeKind::Overlay,
    ),
    (
        "rendercomp-free/starfield",
        "starfield-v1",
        "starfield",
        NativeKind::Overlay,
    ),
    (
        "rendercomp-free/crt-scanlines",
        "crt-scanlines-v1",
        "crt-scanlines",
        NativeKind::Overlay,
    ),
    (
        "rendercomp-free/slide-wipe",
        "slide-wipe-v1",
        "slide-wipe",
        NativeKind::Transition,
    ),
    (
        "rendercomp-free/transition-circle-wipe",
        "transition-circle-wipe-v1",
        "transition-circle-wipe",
        NativeKind::Transition,
    ),
];

pub fn adapter_for(source: &str) -> Option<(&'static str, &'static str)> {
    NATIVE.iter().find(|n| n.0 == source).map(|n| (n.1, n.2))
}

/// The native clip of `source` when it has that kind.
pub fn clip_of(source: &str, kind: NativeKind) -> Option<&'static str> {
    NATIVE
        .iter()
        .find(|n| n.0 == source && n.3 == kind)
        .map(|n| n.2)
}

pub fn resolve<'a>(project: &Project, item: &'a Item) -> Option<&'a str> {
    let ItemBody::Clip {
        clip_id,
        derived_id,
        ..
    } = &item.body
    else {
        return None;
    };
    let Some(id) = derived_id else {
        return clip_id.as_deref();
    };
    let entry = project
        .derived
        .iter()
        .find(|e| e["id"].as_str() == Some(id))?;
    let (adapter, clip) = adapter_for(entry["sourceAsset"].as_str()?)?;
    if entry["nativeAdapter"].as_str() != Some(adapter)
        || clip_id.as_deref().is_some_and(|c| c != clip)
    {
        return None;
    }
    Some(clip)
}

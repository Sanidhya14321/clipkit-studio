//! Explicit native technique adapters. Derived source files are never executed.
use ck_core::{
    Project,
    model::{Item, ItemBody},
};

pub fn adapter_for(source: &str) -> Option<(&'static str, &'static str)> {
    match source {
        "rendercomp-free/glitch-text" => Some(("glitch-text-v1", "glitch-text")),
        _ => None,
    }
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

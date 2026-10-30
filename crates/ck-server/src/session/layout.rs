//! `layout check` (spec 7.4, 8.5): run the layout checker over the project's text layers.
//!
//! A text layer is a clip item with a string `text` prop. Layers with a placement in their props
//! (`box`, `fontSize`) are checked as written; the others are solved together first, as the
//! renderer would place them. The platform comes from the brief.

use ck_assets::layout::{self, LayoutIssue, Placement, Platform, TextBlock};
use ck_core::CkResult;
use ck_core::model::{ItemBody, Project};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Session, is_zero};

/// Type scale used to solve layers that have no placement (the default look's).
const SOLVE_TYPE_SCALE: f64 = 1.25;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LayoutCheckResult {
    pub rev: u64,
    pub platform: Platform,
    /// Text layers checked.
    pub checked: usize,
    /// Text layers that had no placement in their props and were solved for the check.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub solved: usize,
    pub ok: bool,
    pub issues: Vec<LayoutIssue>,
    /// Issues left out of `issues` to keep the reply under its size cap.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub omitted: usize,
}

/// The brief's platform; with none named, the one that fits the project's frame.
pub fn platform(project: &Project) -> Platform {
    let named = project
        .brief
        .fields
        .get("platform")
        .and_then(|f| f.value.as_str());
    let (w, h) = (project.settings.width, project.settings.height);
    match named {
        None if w > h => Platform::Landscape,
        None if w == h => Platform::SquareFeed,
        _ => Platform::from_brief(named),
    }
}

/// The canvas the brief asks for: `aspect` wins over `platform`; neither named means none.
pub fn brief_canvas(project: &Project) -> Option<(u32, u32)> {
    let field = |name: &str| {
        project
            .brief
            .fields
            .get(name)
            .and_then(|f| f.value.as_str())
    };
    match (field("aspect"), field("platform")) {
        (Some(a), _) if ck_assets::layout::aspect_canvas(a).is_some() => {
            ck_assets::layout::aspect_canvas(a)
        }
        (_, Some(p)) => Some(Platform::from_brief(Some(p)).canvas()),
        _ => None,
    }
}

/// Checks the project's text layers.
pub fn check(project: &Project) -> LayoutCheckResult {
    let platform = platform(project);
    let mut placed: Vec<Placement> = Vec::new();
    let mut unplaced: Vec<TextBlock> = Vec::new();
    for item in project.tracks.iter().flat_map(|t| &t.items) {
        let ItemBody::Clip { at, dur, props, .. } = &item.body else {
            continue;
        };
        let Some(block) = layout::text_block(&item.id, *at, *dur, props) else {
            continue;
        };
        match layout::placement_from_props(&block, props) {
            Some(p) => placed.push(p),
            None => unplaced.push(block),
        }
    }
    let solved = unplaced.len();
    placed.extend(layout::solve(platform, SOLVE_TYPE_SCALE, &unplaced));
    let issues = layout::check(platform, &placed);
    LayoutCheckResult {
        rev: project.rev,
        platform,
        checked: placed.len(),
        solved,
        ok: issues.is_empty(),
        issues,
        omitted: 0,
    }
}

impl Session {
    /// `layout check`: read-only.
    pub fn layout_check(&self) -> CkResult<LayoutCheckResult> {
        Ok(check(&self.project()?))
    }
}

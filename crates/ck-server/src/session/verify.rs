//! `verify` (spec 4.3): the deterministic quality gates that apply to a generated video. Every
//! gate runs; the reply lists the gates that ran and the issues each one found.
//!
//! Gates: `schema` (structure and references, clip ids known), `gaps` (base track covered from
//! 0), `overlaps` (no two items at once on a video, audio or caption track; overlay tracks
//! stack by design), `duration` (timeline end matches the brief's `length`), `layout` (safe
//! area, size, collisions: the layout checker), `contrast` (text color against the backdrop
//! under it), `fonts` (at most two families), `motion` (at most 3 layers animating at once),
//! `reuse` (an asset at most `MAX_REUSE` times unless its layers are marked `motif`),
//! `provenance` (every used asset recorded and referenced, every recipe layer traced and
//! licensed) and `placeholders` (no unfilled slot, PLACEHOLDER, lorem ipsum or TODO text).

use std::collections::{BTreeMap, BTreeSet};

use ck_assets::color::{contrast, parse_hex};
use ck_assets::library;
use ck_assets::look::{Look, MIN_TEXT_CONTRAST};
use ck_assets::recipe::{CORE_CLIPS, token_value};
use ck_core::CkResult;
use ck_core::model::{Item, ItemBody, Project, TrackKind};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Session, is_zero};

/// Gates in the order they run.
pub const GATES: &[&str] = &[
    "schema",
    "canvas",
    "gaps",
    "overlaps",
    "duration",
    "layout",
    "contrast",
    "fonts",
    "motion",
    "reuse",
    "provenance",
    "placeholders",
];

/// Smallest gap or overlap reported, in seconds (one frame at 25 fps).
const EPS: f64 = 0.04;
/// Duration tolerance: the larger of 0.5 s and 5% of the brief's length.
const DURATION_ABS: f64 = 0.5;
const DURATION_REL: f64 = 0.05;
/// At most two font families (spec 4.3).
const MAX_FONTS: usize = 2;
/// At most 3 layers animating at the same time (assets/taste/motion.md).
const MAX_ANIMATED: usize = 3;
/// Times one asset may be used before it needs the `motif` mark.
pub const MAX_REUSE: usize = 3;
/// Clip ids that do not animate (they hold the frame under the rest).
const STILL_CLIPS: &[&str] = &["backdrop"];
const PLACEHOLDER_MARKS: &[&str] = &["placeholder", "lorem ipsum", "todo", "tbd", "xxx"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VerifyIssue {
    /// One of `GATES`.
    pub gate: String,
    pub ids: Vec<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VerifyResult {
    pub rev: u64,
    /// Look whose tokens were resolved for contrast and fonts.
    pub look: String,
    pub ok: bool,
    /// Gates that ran and found nothing.
    pub passed: Vec<String>,
    /// Gates that found at least one issue.
    pub failed: Vec<String>,
    pub issues: Vec<VerifyIssue>,
    /// Issues left out of `issues` to keep the reply under its size cap.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub omitted: usize,
    /// With `critic`: the contact sheet for the clipkit-critic subagent (spec 4.3), relative to
    /// the project folder.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contact_sheet: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VerifyRequest {
    /// Also render the critic's contact sheet (6 low-resolution frames at cuts and motion
    /// peaks). Once per pass, when the gates are uncertain.
    #[serde(default)]
    pub critic: bool,
}

struct Issues(Vec<VerifyIssue>);

impl Issues {
    fn add(&mut self, gate: &str, ids: Vec<String>, message: String) {
        self.0.push(VerifyIssue {
            gate: gate.into(),
            ids,
            message,
        });
    }
}

/// Start and end of an item on the timeline; `None` for a voice item without a take.
fn span(project: &Project, item: &Item) -> Option<(f64, f64)> {
    match &item.body {
        ItemBody::Footage {
            src_in,
            src_out,
            at,
            speed,
            ..
        } => Some((*at, at + (src_out - src_in) / speed)),
        ItemBody::Clip { at, dur, .. } => Some((*at, at + dur)),
        ItemBody::Voice { slot_id, at } => {
            let slot = project.voice_slots.iter().find(|s| &s.id == slot_id)?;
            let take = slot
                .takes
                .iter()
                .find(|t| Some(&t.id) == slot.active_take.as_ref())?;
            Some((*at, at + take.duration_sec))
        }
        ItemBody::Audio { at, ranges, .. } => {
            let len: f64 = ranges.iter().map(|r| r.end - r.start).sum();
            Some((*at, at + len))
        }
        ItemBody::Caption { .. } => None,
    }
}

fn clip(item: &Item) -> Option<(Option<&str>, f64, f64, &Value)> {
    match &item.body {
        ItemBody::Clip {
            clip_id,
            at,
            dur,
            props,
            ..
        } => Some((clip_id.as_deref(), *at, at + dur, props)),
        _ => None,
    }
}

/// A prop value with a `$role.name` token resolved against the look.
fn resolve(look: &Look, v: Option<&Value>) -> Option<Value> {
    let v = v?;
    match v.as_str().and_then(|s| s.strip_prefix('$')) {
        Some(token) => token_value(look, token),
        None => Some(v.clone()),
    }
}

fn resolve_str(look: &Look, v: Option<&Value>) -> Option<String> {
    resolve(look, v)?.as_str().map(str::to_owned)
}

fn schema(project: &Project, out: &mut Issues) {
    if let Err(e) = project.validate() {
        out.add("schema", vec![], e.to_string());
    }
    for item in project.items() {
        if let ItemBody::Clip {
            derived_id: Some(id),
            ..
        } = &item.body
            && ck_assets::native_adapter::resolve(project, item).is_none()
        {
            out.add(
                "schema",
                vec![item.id.clone()],
                format!("derived component `{id}` has no registered renderer"),
            );
        }
        if let ItemBody::Clip {
            clip_id: Some(c),
            derived_id: None,
            ..
        } = &item.body
            && !CORE_CLIPS.contains(&c.as_str())
        {
            out.add(
                "schema",
                vec![item.id.clone()],
                format!("clip `{c}` is not a known clip"),
            );
        }
    }
}

fn gaps(project: &Project, out: &mut Issues) {
    let Some(base) = project.tracks.iter().find(|t| t.kind == TrackKind::Video) else {
        if project.items().next().is_some() {
            out.add("gaps", vec![], "there is no base video track".into());
        }
        return;
    };
    let mut spans: Vec<(f64, f64, &str)> = base
        .items
        .iter()
        .filter_map(|i| span(project, i).map(|(a, b)| (a, b, i.id.as_str())))
        .collect();
    spans.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut covered = 0.0_f64;
    let mut prev: Option<&str> = None;
    for (start, end, id) in spans {
        if start > covered + EPS {
            let mut ids: Vec<String> = prev.map(str::to_owned).into_iter().collect();
            ids.push(id.to_owned());
            out.add(
                "gaps",
                ids,
                format!(
                    "base track `{}` is empty from {covered:.2} s to {start:.2} s",
                    base.id
                ),
            );
        }
        if end > covered {
            covered = end;
            prev = Some(id);
        }
    }
}

fn overlaps(project: &Project, out: &mut Issues) {
    for track in project
        .tracks
        .iter()
        .filter(|t| t.kind != TrackKind::Overlay)
    {
        let mut spans: Vec<(f64, f64, &str)> = track
            .items
            .iter()
            .filter_map(|i| span(project, i).map(|(a, b)| (a, b, i.id.as_str())))
            .collect();
        spans.sort_by(|a, b| a.0.total_cmp(&b.0));
        for pair in spans.windows(2) {
            let ((_, a_end, a), (b_start, _, b)) = (pair[0], pair[1]);
            if a_end > b_start + EPS {
                out.add(
                    "overlaps",
                    vec![a.to_owned(), b.to_owned()],
                    format!(
                        "`{a}` and `{b}` overlap on track `{}` by {:.2} s",
                        track.id,
                        a_end - b_start
                    ),
                );
            }
        }
    }
}

fn duration(project: &Project, out: &mut Issues) {
    let Some(length) = project
        .brief
        .fields
        .get("length")
        .and_then(|f| f.value.as_f64())
    else {
        return;
    };
    let end = project
        .items()
        .filter_map(|i| span(project, i))
        .map(|(_, e)| e)
        .fold(0.0_f64, f64::max);
    let tolerance = DURATION_ABS.max(length * DURATION_REL);
    if (end - length).abs() > tolerance {
        out.add(
            "duration",
            vec![],
            format!(
                "timeline ends at {end:.2} s; the brief asks for {length} s (within {tolerance:.2} s)"
            ),
        );
    }
}

fn layout(project: &Project, out: &mut Issues) {
    for issue in super::layout::check(project).issues {
        out.add(
            "layout",
            issue.ids,
            format!("{}: {}", issue.rule, issue.message),
        );
    }
}

/// The backdrop color under a text layer: the video-track clip with a `color` prop that is on
/// screen when the text starts, else the look's background.
fn backdrop(project: &Project, look: &Look, at: f64) -> String {
    project
        .tracks
        .iter()
        .filter(|t| t.kind == TrackKind::Video)
        .flat_map(|t| &t.items)
        .filter_map(clip)
        .filter(|(_, start, end, _)| *start <= at + EPS && at < *end)
        .filter_map(|(_, _, _, props)| {
            resolve_str(look, props.get("ground").or(props.get("color")))
        })
        .next_back()
        .unwrap_or_else(|| look.colors.background.clone())
}

fn contrast_gate(project: &Project, look: &Look, out: &mut Issues) {
    for item in project.items() {
        let Some((_, at, _, props)) = clip(item) else {
            continue;
        };
        if props.get("text").and_then(Value::as_str).is_none() {
            continue;
        }
        let Some(fg) = resolve_str(look, props.get("color")) else {
            continue;
        };
        let bg = backdrop(project, look, at);
        match (parse_hex(&fg), parse_hex(&bg)) {
            (Some(f), Some(b)) => {
                let ratio = contrast(f, b);
                if ratio < MIN_TEXT_CONTRAST {
                    out.add(
                        "contrast",
                        vec![item.id.clone()],
                        format!(
                            "text {fg} on {bg} has contrast {ratio:.2}; at least {MIN_TEXT_CONTRAST}"
                        ),
                    );
                }
            }
            _ => out.add(
                "contrast",
                vec![item.id.clone()],
                format!("cannot read colors {fg:?} on {bg:?} as hex"),
            ),
        }
    }
}

fn fonts(project: &Project, look: &Look, out: &mut Issues) {
    let mut families: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for item in project.items() {
        let Some((_, _, _, props)) = clip(item) else {
            continue;
        };
        if let Some(f) = resolve_str(look, props.get("font")) {
            families.entry(f).or_default().push(item.id.clone());
        }
    }
    if families.len() > MAX_FONTS {
        let names: Vec<String> = families.keys().cloned().collect();
        let ids = families.into_values().flatten().collect();
        out.add(
            "fonts",
            ids,
            format!(
                "{} font families ({}); at most {MAX_FONTS}",
                names.len(),
                names.join(", ")
            ),
        );
    }
}

fn motion(project: &Project, out: &mut Issues) {
    let animated: Vec<(f64, f64, &str)> = project
        .items()
        .filter_map(|i| clip(i).map(|c| (i.id.as_str(), c)))
        .filter(|(_, (c, ..))| !c.is_some_and(|c| STILL_CLIPS.contains(&c)))
        .map(|(id, (_, start, end, _))| (start, end, id))
        .collect();
    let mut reported: BTreeSet<Vec<String>> = BTreeSet::new();
    // The most layers on screen at once is reached at some layer's start.
    for &(t, ..) in &animated {
        let mut ids: Vec<String> = animated
            .iter()
            .filter(|(s, e, _)| *s <= t + EPS && t + EPS < *e)
            .map(|(.., id)| (*id).to_owned())
            .collect();
        ids.sort();
        if ids.len() > MAX_ANIMATED && reported.insert(ids.clone()) {
            out.add(
                "motion",
                ids.clone(),
                format!(
                    "{} layers animate at {t:.2} s; at most {MAX_ANIMATED}",
                    ids.len()
                ),
            );
        }
    }
}

/// Uses per asset: the most layers of that asset sharing one name (one per application).
fn reuse(project: &Project, out: &mut Issues) {
    let mut uses: BTreeMap<(&str, &str), Vec<&str>> = BTreeMap::new();
    for item in project.items() {
        let motif = clip(item).is_some_and(|(.., p)| p.get("motif") == Some(&Value::Bool(true)));
        let asset = item
            .provenance
            .as_ref()
            .map(|p| p.source_asset.as_str())
            .or(item.asset_ref.as_deref());
        if let (Some(asset), false) = (asset, motif) {
            uses.entry((asset, item.name.as_str()))
                .or_default()
                .push(&item.id);
        }
    }
    let mut worst: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for ((asset, _), ids) in uses {
        let w = worst.entry(asset).or_default();
        if ids.len() > w.len() {
            *w = ids;
        }
    }
    for (asset, ids) in worst {
        if ids.len() > MAX_REUSE {
            out.add(
                "reuse",
                ids.iter().map(|s| (*s).to_owned()).collect(),
                format!(
                    "`{asset}` is used {} times; at most {MAX_REUSE} unless marked motif",
                    ids.len()
                ),
            );
        }
    }
}

fn provenance(project: &Project, out: &mut Issues) {
    let mut referenced: BTreeSet<&str> = BTreeSet::new();
    for item in project.items() {
        if let Some(a) = &item.asset_ref {
            referenced.insert(a);
        }
        match &item.provenance {
            Some(p) => {
                referenced.insert(&p.source_asset);
                if p.license.trim().is_empty() || p.hash.trim().is_empty() {
                    out.add(
                        "provenance",
                        vec![item.id.clone()],
                        format!("`{}` has no license or hash recorded", p.source_asset),
                    );
                }
                if !project.assets_used.contains(&p.source_asset) {
                    out.add(
                        "provenance",
                        vec![item.id.clone()],
                        format!("`{}` is not listed in assetsUsed", p.source_asset),
                    );
                }
            }
            None => {
                if clip(item).is_some_and(|(c, ..)| c.is_some()) {
                    out.add(
                        "provenance",
                        vec![item.id.clone()],
                        "clip layer has no provenance".into(),
                    );
                }
            }
        }
    }
    for a in &project.assets_used {
        if !referenced.contains(a.as_str()) {
            out.add(
                "provenance",
                vec![],
                format!("`{a}` is listed as used but no layer references it"),
            );
        }
    }
}

/// True when a text value is an unfilled `{slot}` or contains a placeholder mark as a word.
fn is_placeholder(s: &str) -> bool {
    let t = s.trim();
    if t.len() > 2 && t.starts_with('{') && t.ends_with('}') && !t[1..t.len() - 1].contains(' ') {
        return true;
    }
    let lower = t.to_lowercase();
    PLACEHOLDER_MARKS.iter().any(|m| {
        lower.match_indices(m).any(|(i, _)| {
            let before = lower[..i].chars().next_back();
            let after = lower[i + m.len()..].chars().next();
            !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
        })
    })
}

fn strings<'a>(v: &'a Value, acc: &mut Vec<&'a str>) {
    match v {
        Value::String(s) => acc.push(s),
        Value::Array(a) => a.iter().for_each(|x| strings(x, acc)),
        Value::Object(o) => o.values().for_each(|x| strings(x, acc)),
        _ => {}
    }
}

fn placeholders(project: &Project, out: &mut Issues) {
    for item in project.items() {
        let mut texts = Vec::new();
        if let Some((.., props)) = clip(item) {
            strings(props, &mut texts);
        }
        if let Some(bad) = texts.into_iter().find(|s| is_placeholder(s)) {
            out.add(
                "placeholders",
                vec![item.id.clone()],
                format!("placeholder text left: {bad:?}"),
            );
        }
    }
    for slot in &project.voice_slots {
        if is_placeholder(&slot.text) {
            out.add(
                "placeholders",
                vec![slot.id.clone()],
                "voice slot text is a placeholder".into(),
            );
        }
    }
}

/// The project's frame must match what the brief asks for (same ratio).
fn canvas(project: &Project, out: &mut Issues) {
    let (w, h) = (project.settings.width, project.settings.height);
    if let Some((bw, bh)) = super::layout::brief_canvas(project)
        && u64::from(w) * u64::from(bh) != u64::from(h) * u64::from(bw)
    {
        out.add(
            "canvas",
            vec![ck_core::patch::PROJECT_LAYER.into()],
            format!("project is {w}x{h}, the brief asks for {bw}x{bh}"),
        );
    }
}

/// Runs every gate over the project with the given look.
pub fn verify(project: &Project, look: &Look) -> VerifyResult {
    let mut out = Issues(Vec::new());
    schema(project, &mut out);
    canvas(project, &mut out);
    gaps(project, &mut out);
    overlaps(project, &mut out);
    duration(project, &mut out);
    layout(project, &mut out);
    contrast_gate(project, look, &mut out);
    fonts(project, look, &mut out);
    motion(project, &mut out);
    reuse(project, &mut out);
    provenance(project, &mut out);
    placeholders(project, &mut out);
    let issues = out.0;
    let (failed, passed): (Vec<&str>, Vec<&str>) = GATES
        .iter()
        .partition(|g| issues.iter().any(|i| i.gate == **g));
    VerifyResult {
        rev: project.rev,
        look: look.id.clone(),
        ok: issues.is_empty(),
        passed: passed.into_iter().map(str::to_owned).collect(),
        failed: failed.into_iter().map(str::to_owned).collect(),
        issues,
        omitted: 0,
        contact_sheet: None,
    }
}

impl Session {
    /// `verify`: read-only; with `critic`, also the contact sheet.
    pub fn verify(&self, req: &VerifyRequest) -> CkResult<VerifyResult> {
        let project = self.project()?;
        self.gate(&project, ck_pipeline::stage::Op::RunGates)?;
        self.verify_now(req, project)
    }

    /// The gates with no stage check (handoff and export run them as part of their own step).
    pub(super) fn verify_now(
        &self,
        req: &VerifyRequest,
        project: Project,
    ) -> CkResult<VerifyResult> {
        let look = library::look_get(&library::IdRequest {
            id: super::look::project_look_id(&project),
        })?;
        let mut result = verify(&project, &look);
        if req.critic {
            result.contact_sheet = Some(self.contact_sheet(&project)?);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::is_placeholder;

    #[test]
    fn placeholder_marks_match_whole_words() {
        for bad in [
            "{headline}",
            "PLACEHOLDER",
            "Lorem ipsum dolor",
            "TODO: copy",
            "xxx",
        ] {
            assert!(is_placeholder(bad), "{bad}");
        }
        for good in ["Toddler toys", "Ship video in minutes", "{a b}", "Stand-up"] {
            assert!(!is_placeholder(good), "{good}");
        }
    }
}

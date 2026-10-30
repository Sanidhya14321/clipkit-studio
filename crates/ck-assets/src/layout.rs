//! Deterministic layout solver and checker (spec 7.4 recipe path, 8.5).
//!
//! All geometry is in fractions of the frame (x and widths of frame width, y, heights and type
//! sizes of frame height). Numbers come from the `layout`, `typography` and `platform-specs` taste
//! files. The solver places the text blocks of a scene inside the safe area on the column grid,
//! fits each text (shrink one type step, then wrap, then cut words, never below the label size)
//! and stacks blocks that are on screen together with the minimum gap. The checker reports what a
//! placed layout breaks; `solve` output always passes it unless the blocks cannot fit at all.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Platforms of the platform-specs taste file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Platform {
    VerticalShort,
    SquareFeed,
    Landscape,
    Presentation,
}

/// Safe area insets (fractions of frame width for sides, of frame height for top and bottom).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SafeArea {
    pub top: f64,
    pub bottom: f64,
    pub sides: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Aspect {
    Portrait,
    Square,
    Wide,
}

impl Platform {
    /// Parses a brief platform value; the default (vertical-short) when unknown or silent.
    pub fn from_brief(value: Option<&str>) -> Self {
        match value {
            Some("square-feed" | "instagram-feed" | "linkedin") => Self::SquareFeed,
            Some("landscape" | "youtube" | "web") => Self::Landscape,
            Some("presentation") => Self::Presentation,
            _ => Self::VerticalShort,
        }
    }

    pub fn safe_area(self) -> SafeArea {
        let (top, bottom, sides) = match self {
            Self::VerticalShort => (0.10, 0.20, 0.06),
            Self::SquareFeed => (0.06, 0.06, 0.06),
            Self::Landscape => (0.05, 0.08, 0.05),
            Self::Presentation => (0.05, 0.05, 0.05),
        };
        SafeArea { top, bottom, sides }
    }

    fn aspect(self) -> Aspect {
        match self {
            Self::VerticalShort => Aspect::Portrait,
            Self::SquareFeed => Aspect::Square,
            Self::Landscape | Self::Presentation => Aspect::Wide,
        }
    }

    /// Frame height over frame width.
    fn height_over_width(self) -> f64 {
        match self.aspect() {
            Aspect::Portrait => 16.0 / 9.0,
            Aspect::Square => 1.0,
            Aspect::Wide => 9.0 / 16.0,
        }
    }

    /// Grid columns: 12 for 16:9, 6 for 9:16 and 1:1.
    pub fn columns(self) -> usize {
        if self.aspect() == Aspect::Wide { 12 } else { 6 }
    }
}

/// Column gutter, as a fraction of frame width.
pub const GUTTER: f64 = 0.02;
/// Least gap between stacked text blocks, as a fraction of frame height.
pub const STACK_GAP: f64 = 0.03;
/// Most text blocks on screen at once.
pub const MAX_TEXT_BLOCKS: usize = 3;
/// Average glyph advance in em, used to estimate line length without font metrics.
const GLYPH_EM: f64 = 0.55;
const EPS: f64 = 1e-6;

/// Text roles of the typography taste file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum TextRole {
    Display,
    Title,
    Body,
    Caption,
    Label,
}

impl TextRole {
    /// Type size as a fraction of frame height.
    pub fn size(self, platform: Platform) -> f64 {
        let row = match self {
            Self::Display => [0.075, 0.110, 0.090],
            Self::Title => [0.055, 0.080, 0.065],
            Self::Body => [0.035, 0.050, 0.042],
            Self::Caption => [0.032, 0.045, 0.038],
            Self::Label => [0.024, 0.032, 0.028],
        };
        match platform.aspect() {
            Aspect::Portrait => row[0],
            Aspect::Wide => row[1],
            Aspect::Square => row[2],
        }
    }

    fn line_height(self) -> f64 {
        match self {
            Self::Display | Self::Title => 1.1,
            _ => 1.3,
        }
    }

    fn max_lines(self) -> usize {
        match self {
            Self::Display => 2,
            Self::Title => 3,
            Self::Caption | Self::Label => 2,
            Self::Body => 6,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Align {
    Left,
    Center,
}

/// One text block to place. `at` and `dur` are in seconds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TextBlock {
    pub id: String,
    pub text: String,
    pub role: TextRole,
    pub align: Align,
    pub at: f64,
    pub dur: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    fn intersects(&self, o: &Rect) -> bool {
        self.x < o.x + o.w - EPS
            && o.x < self.x + self.w - EPS
            && self.y < o.y + o.h - EPS
            && o.y < self.y + self.h - EPS
    }
}

/// A placed text block: its box, fitted size and lines, and what fitting did.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Placement {
    pub id: String,
    pub rect: Rect,
    pub size: f64,
    pub align: Align,
    pub at: f64,
    pub dur: f64,
    pub lines: Vec<String>,
    pub shrunk: bool,
    /// Words were dropped to fit; the agent should shorten the text.
    pub cut: bool,
}

/// One rule a placed layout breaks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LayoutIssue {
    /// `safe-area`, `overlap`, `gap`, `too-many-blocks`, `too-small`, `off-grid` or `text-cut`.
    pub rule: String,
    pub ids: Vec<String>,
    pub message: String,
}

/// Places the blocks: full-width on the grid inside the safe area, fitted, and stacked with
/// `STACK_GAP` against every block they share screen time with. Deterministic: same input,
/// same output.
pub fn solve(platform: Platform, type_scale: f64, blocks: &[TextBlock]) -> Vec<Placement> {
    let safe = platform.safe_area();
    let width = 1.0 - 2.0 * safe.sides;
    let label = TextRole::Label.size(platform);
    let mut placed: Vec<Placement> = blocks
        .iter()
        .map(|b| {
            let (size, lines, shrunk, cut) = fit(platform, type_scale, label, width, b);
            Placement {
                id: b.id.clone(),
                rect: Rect {
                    x: safe.sides,
                    y: 0.0,
                    w: width,
                    h: lines.len() as f64 * size * b.role.line_height(),
                },
                size,
                align: b.align,
                at: b.at,
                dur: b.dur,
                lines,
                shrunk,
                cut,
            }
        })
        .collect();
    // Each group of blocks sharing screen time is stacked and centered in the safe area.
    let groups = time_groups(&placed);
    let safe_h = 1.0 - safe.top - safe.bottom;
    for group in groups {
        let total: f64 = group.iter().map(|&i| placed[i].rect.h).sum::<f64>()
            + STACK_GAP * group.len().saturating_sub(1) as f64;
        let mut y = safe.top + ((safe_h - total) / 2.0).max(0.0);
        for i in group {
            placed[i].rect.y = round(y);
            placed[i].rect.h = round(placed[i].rect.h);
            y += placed[i].rect.h + STACK_GAP;
        }
    }
    for p in &mut placed {
        p.rect.x = round(p.rect.x);
        p.rect.w = round(p.rect.w);
        p.size = round(p.size);
    }
    placed
}

/// Reports every rule the placed layout breaks, in a stable order.
pub fn check(platform: Platform, placed: &[Placement]) -> Vec<LayoutIssue> {
    let safe = platform.safe_area();
    let label = TextRole::Label.size(platform);
    let mut issues = Vec::new();
    let issue = |rule: &str, ids: Vec<String>, message: String| LayoutIssue {
        rule: rule.into(),
        ids,
        message,
    };
    let edges = grid_edges(platform);
    for p in placed {
        let r = p.rect;
        if r.x < safe.sides - EPS
            || r.x + r.w > 1.0 - safe.sides + EPS
            || r.y < safe.top - EPS
            || r.y + r.h > 1.0 - safe.bottom + EPS
        {
            issues.push(issue(
                "safe-area",
                vec![p.id.clone()],
                format!("`{}` crosses the safe area", p.id),
            ));
        }
        if p.size < label - EPS {
            issues.push(issue(
                "too-small",
                vec![p.id.clone()],
                format!("`{}` size {} is below the label size {label}", p.id, p.size),
            ));
        }
        let on = |v: f64, xs: &[f64]| xs.iter().any(|e| (e - v).abs() < 1e-3);
        if !on(r.x, &edges.0) || !on(r.x + r.w, &edges.1) {
            issues.push(issue(
                "off-grid",
                vec![p.id.clone()],
                format!("`{}` edges are not on the column grid", p.id),
            ));
        }
        if p.cut {
            issues.push(issue(
                "text-cut",
                vec![p.id.clone()],
                format!("`{}` was cut to fit; shorten the text", p.id),
            ));
        }
    }
    for (i, a) in placed.iter().enumerate() {
        for b in &placed[i + 1..] {
            if !share_time(a, b) {
                continue;
            }
            let ids = vec![a.id.clone(), b.id.clone()];
            if a.rect.intersects(&b.rect) {
                issues.push(issue(
                    "overlap",
                    ids,
                    format!("`{}` and `{}` overlap", a.id, b.id),
                ));
            } else if vertical_gap(&a.rect, &b.rect) < STACK_GAP - 1e-3
                && x_overlap(&a.rect, &b.rect)
            {
                issues.push(issue(
                    "gap",
                    ids,
                    format!(
                        "`{}` and `{}` are closer than {STACK_GAP} of frame height",
                        a.id, b.id
                    ),
                ));
            }
        }
    }
    for group in time_groups(placed) {
        if group.len() > MAX_TEXT_BLOCKS {
            issues.push(issue(
                "too-many-blocks",
                group.iter().map(|&i| placed[i].id.clone()).collect(),
                format!(
                    "{} text blocks on screen at once, at most {MAX_TEXT_BLOCKS}",
                    group.len()
                ),
            ));
        }
    }
    if safe_overflow(platform, placed) {
        issues.push(issue(
            "safe-area",
            Vec::new(),
            "the stacked text is taller than the safe area; split the scene".into(),
        ));
    }
    issues
}

/// The text block of a clip layer: its string `text` prop, with the role from a `role` prop
/// (else `caption` when `size` is `small`, else `title`) and the align from `align` (else
/// center). `None` when the layer shows no text.
pub fn text_block(id: &str, at: f64, dur: f64, props: &serde_json::Value) -> Option<TextBlock> {
    let text = props.get("text")?.as_str()?.trim();
    if text.is_empty() {
        return None;
    }
    let role = props
        .get("role")
        .and_then(|r| serde_json::from_value(r.clone()).ok())
        .unwrap_or(
            if props.get("size").and_then(|s| s.as_str()) == Some("small") {
                TextRole::Caption
            } else {
                TextRole::Title
            },
        );
    let align = props
        .get("align")
        .and_then(|a| serde_json::from_value(a.clone()).ok())
        .unwrap_or(Align::Center);
    Some(TextBlock {
        id: id.into(),
        text: text.into(),
        role,
        align,
        at,
        dur,
    })
}

/// The placement written into a layer's props (`box`, `fontSize`, `lines`, `textCut`), or
/// `None` when the layer has no `box` or `fontSize` yet.
pub fn placement_from_props(block: &TextBlock, props: &serde_json::Value) -> Option<Placement> {
    let rect: Rect = serde_json::from_value(props.get("box")?.clone()).ok()?;
    let size = props.get("fontSize")?.as_f64()?;
    let lines = props
        .get("lines")
        .and_then(|l| serde_json::from_value(l.clone()).ok())
        .unwrap_or_default();
    Some(Placement {
        id: block.id.clone(),
        rect,
        size,
        align: block.align,
        at: block.at,
        dur: block.dur,
        lines,
        shrunk: false,
        cut: props.get("textCut").and_then(|c| c.as_bool()) == Some(true),
    })
}

/// Writes a placement into a layer's props (the inverse of `placement_from_props`).
pub fn write_props(p: &Placement, props: &mut serde_json::Map<String, serde_json::Value>) {
    use serde_json::json;
    props.insert("box".into(), json!(p.rect));
    props.insert("fontSize".into(), json!(p.size));
    props.insert("lines".into(), json!(p.lines));
    if p.cut {
        props.insert("textCut".into(), json!(true));
    } else {
        props.remove("textCut");
    }
}

fn safe_overflow(platform: Platform, placed: &[Placement]) -> bool {
    let safe = platform.safe_area();
    time_groups(placed).iter().any(|g| {
        let total: f64 = g.iter().map(|&i| placed[i].rect.h).sum::<f64>()
            + STACK_GAP * g.len().saturating_sub(1) as f64;
        total > 1.0 - safe.top - safe.bottom + EPS
    })
}

/// Fits one block: base size, then one type step smaller, then cut words; never below `label`.
fn fit(
    platform: Platform,
    type_scale: f64,
    label: f64,
    width: f64,
    b: &TextBlock,
) -> (f64, Vec<String>, bool, bool) {
    let base = b.role.size(platform);
    let max = b.role.max_lines();
    let lines = wrap(&b.text, per_line(platform, base, width));
    if lines.len() <= max {
        return (base, lines, false, false);
    }
    let step = if type_scale > 1.0 { type_scale } else { 1.0 };
    let small = (base / step).max(label);
    let shrunk = small < base;
    let mut lines = wrap(&b.text, per_line(platform, small, width));
    let cut = lines.len() > max;
    lines.truncate(max);
    (small, lines, shrunk, cut)
}

fn per_line(platform: Platform, size: f64, width: f64) -> usize {
    let glyph = size * GLYPH_EM * platform.height_over_width();
    ((width / glyph).floor() as usize).max(1)
}

/// Greedy word wrap; a word longer than a line takes a line of its own.
fn wrap(text: &str, per_line: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > per_line {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

/// Left and right column edges.
fn grid_edges(platform: Platform) -> (Vec<f64>, Vec<f64>) {
    let sides = platform.safe_area().sides;
    let n = platform.columns();
    let col = (1.0 - 2.0 * sides - GUTTER * (n - 1) as f64) / n as f64;
    let lefts: Vec<f64> = (0..n).map(|i| sides + i as f64 * (col + GUTTER)).collect();
    let rights = lefts.iter().map(|l| l + col).collect();
    (lefts, rights)
}

fn share_time(a: &Placement, b: &Placement) -> bool {
    a.at < b.at + b.dur - EPS && b.at < a.at + a.dur - EPS
}

fn x_overlap(a: &Rect, b: &Rect) -> bool {
    a.x < b.x + b.w - EPS && b.x < a.x + a.w - EPS
}

fn vertical_gap(a: &Rect, b: &Rect) -> f64 {
    (b.y - (a.y + a.h)).max(a.y - (b.y + b.h))
}

/// Groups of blocks connected by shared screen time, each in input order; groups ordered by
/// their first block.
fn time_groups(placed: &[Placement]) -> Vec<Vec<usize>> {
    let mut group_of: Vec<usize> = (0..placed.len()).collect();
    for i in 0..placed.len() {
        for j in 0..i {
            if share_time(&placed[i], &placed[j]) {
                let (from, to) = (group_of[i], group_of[j]);
                for g in &mut group_of {
                    if *g == from {
                        *g = to;
                    }
                }
            }
        }
    }
    let mut groups: Vec<(usize, Vec<usize>)> = Vec::new();
    for (i, g) in group_of.iter().enumerate() {
        match groups.iter_mut().find(|(k, _)| k == g) {
            Some((_, v)) => v.push(i),
            None => groups.push((*g, vec![i])),
        }
    }
    groups.into_iter().map(|(_, v)| v).collect()
}

fn round(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(id: &str, text: &str, role: TextRole, at: f64, dur: f64) -> TextBlock {
        TextBlock {
            id: id.into(),
            text: text.into(),
            role,
            align: Align::Center,
            at,
            dur,
        }
    }

    #[test]
    fn solved_layout_passes_check_on_every_platform() {
        let blocks = [
            block("t", "Ship faster with less", TextRole::Display, 0.0, 3.0),
            block(
                "b",
                "Three steps, one command, no setup",
                TextRole::Body,
                0.5,
                2.5,
            ),
            block("c", "Try it today", TextRole::Caption, 1.0, 2.0),
            block("later", "Next scene title", TextRole::Title, 3.0, 2.0),
        ];
        for p in [
            Platform::VerticalShort,
            Platform::SquareFeed,
            Platform::Landscape,
            Platform::Presentation,
        ] {
            let placed = solve(p, 1.25, &blocks);
            assert_eq!(check(p, &placed), Vec::new(), "{p:?}");
            assert_eq!(placed, solve(p, 1.25, &blocks), "deterministic");
            // Stacked in input order with the gap.
            assert!(placed[1].rect.y >= placed[0].rect.y + placed[0].rect.h + STACK_GAP - 1e-3);
        }
    }

    #[test]
    fn fitting_shrinks_then_cuts_never_below_label() {
        let p = Platform::VerticalShort;
        let long = "word ".repeat(30);
        let placed = solve(p, 1.25, &[block("d", &long, TextRole::Display, 0.0, 1.0)]);
        assert!(placed[0].shrunk && placed[0].cut);
        assert_eq!(placed[0].lines.len(), 2);
        assert!(placed[0].size >= TextRole::Label.size(p));
        assert_eq!(check(p, &placed)[0].rule, "text-cut");

        let short = solve(p, 1.25, &[block("s", "Hello", TextRole::Display, 0.0, 1.0)]);
        assert!(!short[0].shrunk && !short[0].cut);
        assert_eq!(short[0].lines, vec!["Hello".to_string()]);
    }

    #[test]
    fn check_reports_each_rule() {
        let p = Platform::Landscape;
        let mut placed = solve(
            p,
            1.25,
            &[
                block("a", "One", TextRole::Title, 0.0, 2.0),
                block("b", "Two", TextRole::Title, 0.0, 2.0),
            ],
        );
        placed[1].rect.y = placed[0].rect.y; // overlap
        placed[0].rect.x = 0.0; // safe area and grid
        placed[0].size = 0.01; // too small
        let rules: Vec<_> = check(p, &placed).into_iter().map(|i| i.rule).collect();
        assert_eq!(rules, ["safe-area", "too-small", "off-grid", "overlap"]);

        let mut close = solve(
            p,
            1.25,
            &[
                block("a", "One", TextRole::Body, 0.0, 2.0),
                block("b", "Two", TextRole::Body, 0.0, 2.0),
            ],
        );
        close[1].rect.y = close[0].rect.y + close[0].rect.h + 0.01;
        assert_eq!(check(p, &close)[0].rule, "gap");

        let four: Vec<_> = (0..4)
            .map(|i| block(&format!("x{i}"), "Hi", TextRole::Label, 0.0, 1.0))
            .collect();
        let rules: Vec<_> = check(p, &solve(p, 1.25, &four))
            .into_iter()
            .map(|i| i.rule)
            .collect();
        assert_eq!(rules, ["too-many-blocks"]);
    }

    #[test]
    fn blocks_apart_in_time_do_not_interact() {
        let p = Platform::SquareFeed;
        let placed = solve(
            p,
            1.25,
            &[
                block("a", "First", TextRole::Display, 0.0, 1.0),
                block("b", "Second", TextRole::Display, 1.0, 1.0),
            ],
        );
        assert_eq!(placed[0].rect.y, placed[1].rect.y);
        assert!(check(p, &placed).is_empty());
    }

    #[test]
    fn props_round_trip_and_roles() {
        use serde_json::json;
        assert!(text_block("x", 0.0, 1.0, &json!({ "color": "#fff" })).is_none());
        assert!(text_block("x", 0.0, 1.0, &json!({ "text": "  " })).is_none());
        let small = text_block("s", 0.0, 1.0, &json!({ "text": "Hi", "size": "small" }));
        assert_eq!(small.map(|b| b.role), Some(TextRole::Caption));
        let b = text_block("d", 0.5, 2.0, &json!({ "text": "Ship", "role": "display" }));
        let Some(b) = b else { panic!("text block") };
        assert_eq!((b.role, b.align), (TextRole::Display, Align::Center));
        assert!(placement_from_props(&b, &json!({ "text": "Ship" })).is_none());

        let placed = solve(Platform::Landscape, 1.25, std::slice::from_ref(&b));
        let mut props = serde_json::Map::new();
        write_props(&placed[0], &mut props);
        let back = placement_from_props(&b, &serde_json::Value::Object(props));
        let Some(back) = back else {
            panic!("placement")
        };
        assert_eq!(
            (back.rect, back.size, &back.lines),
            (placed[0].rect, placed[0].size, &placed[0].lines)
        );
    }

    #[test]
    fn platform_from_brief_defaults_to_vertical_short() {
        assert_eq!(Platform::from_brief(None), Platform::VerticalShort);
        assert_eq!(Platform::from_brief(Some("landscape")), Platform::Landscape);
        assert_eq!(Platform::Landscape.columns(), 12);
    }
}

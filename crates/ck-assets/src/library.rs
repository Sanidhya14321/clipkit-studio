//! The starter library built into the binary, and the read ops over it: `ck taste search|read`
//! and `ck look list|get` (spec 6.2, 7.1). Embedding keeps an installed `ck` independent of the
//! repo's `assets/` folder; a test keeps the table in step with that folder.

use std::path::Path;

use ck_core::{CkError, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::look::{Look, MotionPersonality};
use crate::recipe::{Recipe, Slot, SlotKind};
use crate::taste::TasteFile;

/// Output caps in bytes of compact JSON, shared by each command and its MCP twin.
pub mod caps {
    pub const TASTE_SEARCH: usize = 2 * 1024;
    pub const TASTE_READ: usize = 3 * 1024;
    pub const LOOK_LIST: usize = 2 * 1024;
    pub const LOOK_GET: usize = 2 * 1024;
    pub const RECIPE_SEARCH: usize = 4 * 1024;
    pub const RECIPE_CARD: usize = 6 * 1024;
}

macro_rules! embed {
    ($dir:literal, $ext:literal, [$($name:literal),* $(,)?]) => {
        &[$(($name, include_str!(concat!("../../../assets/", $dir, "/", $name, ".", $ext)))),*]
    };
}

/// (id, text) of every built-in taste file.
pub const TASTE_FILES: &[(&str, &str)] = embed!(
    "taste",
    "md",
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

/// (id, text) of every built-in look.
pub const LOOK_FILES: &[(&str, &str)] = embed!(
    "looks",
    "json",
    [
        "bold-social",
        "calm-documentary",
        "cinematic",
        "clean-product",
    ]
);

/// (id, text) of every built-in recipe (spec 7.5).
pub const RECIPE_FILES: &[(&str, &str)] = embed!(
    "recipes",
    "json",
    [
        "before-after",
        "hook-title",
        "kinetic-list",
        "outro-cta",
        "product-reveal",
        "quote-testimonial",
        "section-card",
        "stat-callout",
        "step-explainer",
        "talking-head",
    ]
);

/// Default and largest number of taste search hits.
pub const DEFAULT_HITS: usize = 5;
pub const MAX_HITS: usize = 12;

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TasteSearchRequest {
    /// Words to look for in ids, titles, tags and bodies; empty lists every file.
    #[serde(default)]
    pub query: String,
    /// Most hits to return (default 5, at most 12).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TasteHit {
    pub id: String,
    pub title: String,
    pub tags: Vec<String>,
    pub score: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TasteSearchResult {
    pub hits: Vec<TasteHit>,
    /// Files that matched before the limit.
    pub total: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IdRequest {
    pub id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LookListRequest {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LookSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub personality: MotionPersonality,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LookListResult {
    pub looks: Vec<LookSummary>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecipeSearchRequest {
    /// Words to look for in ids, names, tags and use-when lines; empty lists every recipe.
    #[serde(default)]
    pub query: String,
    /// Most hits to return (default 5, at most 12).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

/// One recipe in short: enough to pick it and fill its slots without reading the card.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecipeHit {
    pub id: String,
    pub name: String,
    pub use_when: String,
    /// Default scene duration in seconds.
    pub duration: f64,
    /// `name:kind`, with `*` when required and `<=N` for the character limit.
    pub slots: Vec<String>,
    pub score: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecipeSearchResult {
    pub hits: Vec<RecipeHit>,
    /// Recipes that matched before the limit.
    pub total: usize,
}

/// Every built-in taste file, parsed and validated, sorted by id.
pub fn taste() -> Result<Vec<TasteFile>, CkError> {
    TASTE_FILES
        .iter()
        .map(|(id, text)| TasteFile::parse(&virtual_path("taste", id, "md"), text))
        .collect()
}

/// Every built-in look, parsed and validated, sorted by id.
pub fn looks() -> Result<Vec<Look>, CkError> {
    LOOK_FILES
        .iter()
        .map(|(id, text)| Look::parse(&virtual_path("looks", id, "json"), text))
        .collect()
}

/// Every built-in recipe, parsed and validated, sorted by id.
pub fn recipes() -> Result<Vec<Recipe>, CkError> {
    RECIPE_FILES
        .iter()
        .map(|(id, text)| Recipe::parse(&virtual_path("recipes", id, "json"), text))
        .collect()
}

fn virtual_path(dir: &str, id: &str, ext: &str) -> std::path::PathBuf {
    Path::new("assets").join(dir).join(format!("{id}.{ext}"))
}

fn query_words(query: &str) -> Vec<String> {
    query
        .split(|c: char| !c.is_alphanumeric() && c != '-')
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// 3 points per word in `head`, else 1 if in `body`.
fn score(words: &[String], head: &str, body: &str) -> u32 {
    let (head, body) = (head.to_lowercase(), body.to_lowercase());
    words
        .iter()
        .map(|w| {
            if head.contains(w.as_str()) {
                3
            } else {
                u32::from(body.contains(w.as_str()))
            }
        })
        .sum()
}

/// Ranks taste files by query words: 3 points per word in the id, title or tags, 1 in the body.
/// Ties keep id order.
pub fn taste_search(req: &TasteSearchRequest) -> Result<TasteSearchResult, CkError> {
    let words = query_words(&req.query);
    let mut hits: Vec<TasteHit> = taste()?
        .into_iter()
        .filter_map(|f| {
            let head = format!("{} {} {}", f.id, f.title, f.tags.join(" "));
            let score = score(&words, &head, &f.body);
            (words.is_empty() || score > 0).then_some(TasteHit {
                id: f.id,
                title: f.title,
                tags: f.tags,
                score,
            })
        })
        .collect();
    hits.sort_by_key(|h| std::cmp::Reverse(h.score));
    let total = hits.len();
    hits.truncate(req.limit.unwrap_or(DEFAULT_HITS).clamp(1, MAX_HITS));
    Ok(TasteSearchResult { hits, total })
}

/// One taste file in full.
pub fn taste_read(req: &IdRequest) -> Result<TasteFile, CkError> {
    taste()?
        .into_iter()
        .find(|f| f.id == req.id)
        .ok_or_else(|| not_found("taste file", &req.id, "ck taste search"))
}

/// Every look in short.
pub fn look_list(_req: &LookListRequest) -> Result<LookListResult, CkError> {
    Ok(LookListResult {
        looks: looks()?
            .into_iter()
            .map(|l| LookSummary {
                id: l.id,
                name: l.name,
                description: l.description,
                personality: l.motion.personality,
            })
            .collect(),
    })
}

/// One look with all its tokens.
pub fn look_get(req: &IdRequest) -> Result<Look, CkError> {
    looks()?
        .into_iter()
        .find(|l| l.id == req.id)
        .ok_or_else(|| not_found("look", &req.id, "ck look list"))
}

/// Ranks recipes by query words: 3 points per word in the id, name or tags, 1 in the
/// description or use-when line. Ties keep id order.
pub fn recipe_search(req: &RecipeSearchRequest) -> Result<RecipeSearchResult, CkError> {
    let words = query_words(&req.query);
    let mut hits: Vec<RecipeHit> = recipes()?
        .into_iter()
        .filter_map(|r| {
            let head = format!("{} {} {}", r.id, r.name, r.tags.join(" "));
            let body = format!("{} {}", r.description, r.use_when);
            let score = score(&words, &head, &body);
            (words.is_empty() || score > 0).then(|| RecipeHit {
                slots: r.slots.iter().map(slot_line).collect(),
                id: r.id,
                name: r.name,
                use_when: r.use_when,
                duration: r.duration.default,
                score,
            })
        })
        .collect();
    hits.sort_by_key(|h| std::cmp::Reverse(h.score));
    let total = hits.len();
    hits.truncate(req.limit.unwrap_or(DEFAULT_HITS).clamp(1, MAX_HITS));
    Ok(RecipeSearchResult { hits, total })
}

fn slot_line(s: &Slot) -> String {
    let kind = match s.kind {
        SlotKind::Text => "text",
        SlotKind::Number => "number",
        SlotKind::List => "list",
        SlotKind::Source => "source",
    };
    let mut line = format!("{}{}:{kind}", s.name, if s.required { "*" } else { "" });
    if let (Some(lo), Some(hi)) = (s.min_items, s.max_items) {
        line.push_str(&format!("[{lo}-{hi}]"));
    }
    if let Some(n) = s.max_chars {
        line.push_str(&format!("<={n}"));
    }
    line
}

/// One recipe in full: slots, layers and their props.
pub fn recipe_card(req: &IdRequest) -> Result<Recipe, CkError> {
    recipe_with_text(&req.id).map(|(r, _)| r)
}

/// One recipe and its file text (the text gives the provenance hash).
pub fn recipe_with_text(id: &str) -> Result<(Recipe, &'static str), CkError> {
    let text = RECIPE_FILES
        .iter()
        .find(|(rid, _)| *rid == id)
        .map(|(_, t)| *t)
        .ok_or_else(|| not_found("recipe", id, "ck recipe search"))?;
    Ok((
        Recipe::parse(&virtual_path("recipes", id, "json"), text)?,
        text,
    ))
}

fn not_found(what: &str, id: &str, list: &str) -> CkError {
    CkError::new(
        ErrorCode::NotFound,
        format!("no {what} `{id}`; {list} lists them"),
    )
}

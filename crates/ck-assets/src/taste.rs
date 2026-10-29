//! Taste files: short direction notes with concrete numbers (spec 7.1, 7.5).
//!
//! Format: a front matter block (`id`, `title`, `tags: [a, b]`) between `---` lines, then a
//! markdown body. The id must match the file name.

use std::path::Path;

use ck_core::CkError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{asset_error, list_files, read_file, stem};

/// Most bytes a taste file may have; about 600 tokens at 4 bytes per token, so reading the three
/// files the skill allows stays near the spec 9.2 retrieval budget.
pub const MAX_TASTE_BYTES: usize = 2400;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TasteFile {
    pub id: String,
    pub title: String,
    pub tags: Vec<String>,
    /// Markdown body after the front matter.
    pub body: String,
}

impl TasteFile {
    /// Parses and validates one taste file; `path` names the file in errors and gives the id.
    pub fn parse(path: &Path, text: &str) -> Result<Self, CkError> {
        if text.len() > MAX_TASTE_BYTES {
            return Err(asset_error(
                path,
                format!("{} bytes, over the {MAX_TASTE_BYTES} byte cap", text.len()),
            ));
        }
        let rest = text
            .strip_prefix("---\n")
            .ok_or_else(|| asset_error(path, "missing front matter"))?;
        let (front, body) = rest
            .split_once("\n---\n")
            .ok_or_else(|| asset_error(path, "unclosed front matter"))?;
        let (mut id, mut title, mut tags) = (None, None, None);
        for line in front.lines() {
            let (key, value) = line
                .split_once(':')
                .ok_or_else(|| asset_error(path, format!("bad front matter line `{line}`")))?;
            let value = value.trim();
            match key.trim() {
                "id" => id = Some(value.to_owned()),
                "title" => title = Some(value.to_owned()),
                "tags" => tags = Some(parse_list(value).ok_or_else(|| asset_error(path, "tags"))?),
                other => return Err(asset_error(path, format!("unknown field `{other}`"))),
            }
        }
        let file = Self {
            id: id.ok_or_else(|| asset_error(path, "missing id"))?,
            title: title.ok_or_else(|| asset_error(path, "missing title"))?,
            tags: tags.ok_or_else(|| asset_error(path, "missing tags"))?,
            body: body.trim().to_owned(),
        };
        if file.id != stem(path) {
            return Err(asset_error(
                path,
                format!("id `{}` must match the file name", file.id),
            ));
        }
        if file.title.is_empty() || file.tags.is_empty() || file.body.is_empty() {
            return Err(asset_error(path, "title, tags and body must not be empty"));
        }
        if !file.body.chars().any(|c| c.is_ascii_digit()) {
            return Err(asset_error(path, "body has no concrete numbers"));
        }
        Ok(file)
    }
}

/// Parses `[a, b, c]` into non-empty items.
pub(crate) fn parse_list(value: &str) -> Option<Vec<String>> {
    let inner = value.strip_prefix('[')?.strip_suffix(']')?;
    let items: Vec<String> = inner.split(',').map(|s| s.trim().to_owned()).collect();
    items.iter().all(|s| !s.is_empty()).then_some(items)
}

/// Loads every `*.md` taste file in `dir`, sorted by id.
pub fn load_taste(dir: &Path) -> Result<Vec<TasteFile>, CkError> {
    list_files(dir, "md")?
        .iter()
        .map(|path| TasteFile::parse(path, &read_file(path)?))
        .collect()
}

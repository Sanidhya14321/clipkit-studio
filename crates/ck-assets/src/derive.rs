//! Derive (spec 7.4 rung 3): a project-local copy of an asset changed by codemods guided by the
//! AST: colour literals become named tokens in one `ckTokens` object (so a look can re-theme
//! them), each exported component's root element is instrumented with `data-ck-layer` for
//! hover-select, and a props JSON schema is generated from the typed props. Provenance (source
//! asset, hash, mode, adaptations, license) is written beside it. The copy is never run here;
//! type checking and the first render belong to the render worker (Phase 9).

use std::path::{Path, PathBuf};

use ck_core::{CkError, CkResult, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::ast::{AstMap, extract};
use crate::card::{Card, LicenseClass, LicenseMode, Runtime};
use crate::hub::Hub;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeriveProvenance {
    pub source_asset: String,
    /// SHA-256 of the original file.
    pub hash: String,
    pub mode: String,
    pub adaptations: Vec<String>,
    pub license: String,
    pub license_class: LicenseClass,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attribution: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Derived {
    /// Folder under `.clipkit/derived/`.
    pub name: String,
    /// Files written, relative to the project folder.
    pub files: Vec<String>,
    pub provenance: DeriveProvenance,
    /// Colour literals moved into `ckTokens`.
    pub tokens: Vec<(String, String)>,
}

fn io(path: &Path, e: impl std::fmt::Display) -> CkError {
    CkError::new(ErrorCode::Storage, format!("{}: {e}", path.display()))
}

/// `#rgb`, `#rrggbb` or `#rrggbbaa` string literals (quoted) in `src`, in order of first use.
pub fn colour_literals(src: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let b = src.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let q = b[i];
        if (q == b'"' || q == b'\'') && b.get(i + 1) == Some(&b'#') {
            let start = i + 2;
            let mut j = start;
            while j < b.len() && b[j].is_ascii_hexdigit() {
                j += 1;
            }
            let n = j - start;
            if b.get(j) == Some(&q) && matches!(n, 3 | 6 | 8) {
                let lit = src[i + 1..j].to_lowercase();
                if !out.contains(&lit) {
                    out.push(lit);
                }
                i = j + 1;
                continue;
            }
        }
        i += 1;
    }
    out
}

/// Replaces quoted colour literals with `ckTokens.<name>` and returns the token table.
fn tokenize_colours(src: &str) -> (String, Vec<(String, String)>) {
    let colours = colour_literals(src);
    let table: Vec<(String, String)> = colours
        .iter()
        .enumerate()
        .map(|(n, c)| (format!("color{}", n + 1), c.clone()))
        .collect();
    let mut out = src.to_owned();
    for (name, value) in &table {
        for q in ['"', '\''] {
            let lit = format!("{q}{value}{q}");
            let upper = format!("{q}{}{q}", value.to_uppercase());
            out = out
                .replace(&lit, &format!("ckTokens.{name}"))
                .replace(&upper, &format!("ckTokens.{name}"));
        }
    }
    (out, table)
}

/// Adds `data-ck-layer={layerId}` to the first JSX element of each exported component (the
/// element its `return` or arrow body starts with), using the AST's symbol line ranges.
fn instrument(src: &str, map: &AstMap) -> (String, usize) {
    let comps: Vec<(u32, u32)> = map
        .outline
        .iter()
        .filter(|s| {
            s.exported
                && matches!(s.kind.as_str(), "const" | "function" | "default")
                && s.name
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_uppercase())
        })
        .map(|s| (s.start_line, s.end_line))
        .collect();
    let mut lines: Vec<String> = src.lines().map(str::to_owned).collect();
    let mut done = 0;
    for (start, end) in comps {
        'found: for l in (start as usize - 1)..(end as usize).min(lines.len()) {
            let line = &lines[l];
            let mut from = 0;
            while let Some(p) = line[from..].find('<') {
                let at = from + p;
                let rest = &line[at + 1..];
                let name_len = rest
                    .find(|c: char| !(c.is_ascii_alphanumeric() || c == '.' || c == '_'))
                    .unwrap_or(rest.len());
                let starts_tag = rest.chars().next().is_some_and(|c| c.is_ascii_alphabetic());
                let before = line[..at].trim_end();
                let jsx_context = before.is_empty()
                    || before.ends_with('(')
                    || before.ends_with("return")
                    || before.ends_with("=>");
                if starts_tag && jsx_context && name_len > 0 {
                    let insert = at + 1 + name_len;
                    lines[l].insert_str(insert, " data-ck-layer={ckLayerId}");
                    done += 1;
                    break 'found;
                }
                from = at + 1;
            }
        }
    }
    let mut out = lines.join("\n");
    if src.ends_with('\n') {
        out.push('\n');
    }
    (out, done)
}

/// A JSON schema for the first props type (types as written become descriptions).
fn props_schema(map: &AstMap) -> Value {
    let Some(p) = map.props.first() else {
        return json!({ "type": "object", "properties": {} });
    };
    let ty = |t: &str| match t.trim() {
        "string" => json!({ "type": "string" }),
        "number" => json!({ "type": "number" }),
        "boolean" => json!({ "type": "boolean" }),
        other => json!({ "description": other }),
    };
    let props: serde_json::Map<String, Value> = p
        .fields
        .iter()
        .map(|f| (f.name.clone(), ty(&f.ty)))
        .collect();
    let required: Vec<&str> = p
        .fields
        .iter()
        .filter(|f| !f.optional)
        .map(|f| f.name.as_str())
        .collect();
    json!({ "title": p.name, "type": "object", "properties": props, "required": required })
}

/// Derives `id` into `<project>/.clipkit/derived/<name>/`. Refused for reference-only assets and
/// for licenses the mode does not allow.
pub fn derive(
    hub: &Hub,
    project: &Path,
    id: &str,
    name: &str,
    mode: LicenseMode,
) -> CkResult<Derived> {
    if name.is_empty()
        || !name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Err(CkError::new(
            ErrorCode::Request,
            format!("derived name {name:?} must be lowercase letters, digits and dashes"),
        ));
    }
    let usable = hub
        .sources()?
        .into_iter()
        .any(|s| s.usable() && id.starts_with(&format!("{}/", s.id)));
    let card_path = hub.assets().join(id).join("asset.md");
    if !usable || !card_path.is_file() {
        return Err(CkError::new(
            ErrorCode::NotFound,
            format!("no asset {id:?}"),
        ));
    }
    let card = Card::parse(
        &card_path,
        &std::fs::read_to_string(&card_path).map_err(|e| io(&card_path, e))?,
    )?;
    if card.runtime == Runtime::ReferenceOnly {
        return Err(CkError::new(
            ErrorCode::Request,
            format!("{id} is reference-only: re-implement it natively"),
        ));
    }
    if !mode.allows(card.license_class) {
        return Err(CkError::new(
            ErrorCode::Request,
            format!(
                "{id} is {} and the license mode is {}",
                card.license_class.as_str(),
                mode.as_str()
            ),
        )
        .with_hint("Pick another asset, or change the brief's licenseMode."));
    }
    let ast_path = hub.assets().join(id).join("ast.json");
    let map: AstMap =
        serde_json::from_str(&std::fs::read_to_string(&ast_path).map_err(|e| io(&ast_path, e))?)
            .map_err(|e| io(&ast_path, e))?;
    let file = Path::new(&map.file)
        .file_name()
        .map(PathBuf::from)
        .unwrap_or_default();
    let src_path = hub.assets().join(id).join("src").join(&file);
    let original = std::fs::read_to_string(&src_path).map_err(|e| io(&src_path, e))?;

    let (coloured, tokens) = tokenize_colours(&original);
    // Line ranges are unchanged by the colour codemod, so the map still applies.
    let (instrumented, marked) = instrument(&coloured, &extract(&map.file, &coloured));
    let mut header = format!(
        "// Derived by ClipKit from {id} ({}, {}). Project-local copy; see provenance.json.\n",
        card.license, card.source_commit
    );
    header.push_str("/** Set by the composition so hover-select can find this layer. */\n");
    header.push_str("declare const ckLayerId: string;\n");
    if !tokens.is_empty() {
        header.push_str("/** Colours lifted from the original; a look re-themes them here. */\n");
        header.push_str("export const ckTokens = {\n");
        for (n, v) in &tokens {
            header.push_str(&format!("  {n}: \"{v}\",\n"));
        }
        header.push_str("};\n");
    }
    let body = format!("{header}\n{instrumented}");

    let mut adaptations = Vec::new();
    if !tokens.is_empty() {
        adaptations.push(format!("colours->tokens ({})", tokens.len()));
    }
    if marked > 0 {
        adaptations.push(format!("instrumented ({marked} roots)"));
    }
    adaptations.push("props schema".to_owned());
    let provenance = DeriveProvenance {
        source_asset: id.to_owned(),
        hash: Sha256::digest(original.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect(),
        mode: "derive".into(),
        adaptations,
        license: card.license.clone(),
        license_class: card.license_class,
        attribution: card.attribution.clone(),
    };
    let dir = project.join(".clipkit").join("derived").join(name);
    std::fs::create_dir_all(&dir).map_err(|e| io(&dir, e))?;
    let write = |n: &Path, bytes: &[u8]| ck_core::storage::atomic_write(&dir.join(n), bytes);
    write(&file, body.as_bytes())?;
    let schema = serde_json::to_vec_pretty(&props_schema(&map))
        .map_err(|e| CkError::new(ErrorCode::Server, e.to_string()))?;
    write(Path::new("props.schema.json"), &schema)?;
    let prov = serde_json::to_vec_pretty(&provenance)
        .map_err(|e| CkError::new(ErrorCode::Server, e.to_string()))?;
    write(Path::new("provenance.json"), &prov)?;
    let rel = |n: &str| format!(".clipkit/derived/{name}/{n}");
    Ok(Derived {
        name: name.to_owned(),
        files: vec![
            rel(&file.to_string_lossy()),
            rel("props.schema.json"),
            rel("provenance.json"),
        ],
        provenance,
        tokens,
    })
}

#[cfg(test)]
mod tests;

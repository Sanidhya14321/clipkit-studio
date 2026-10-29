//! `ck sources ingest` (spec 7.1, 7.2): turn a vetted source into cards. Every exported
//! component (PascalCase function or const) of its TS/JS files becomes one asset folder:
//! `asset.md` (the card), `ast.json` (the file's symbol map) and `src/` (the original file,
//! read-only). Then the source's SOURCE.md and INDEX.md and the hub's catalog.jsonl are
//! rebuilt. Parsing only; nothing is run. Owner notes in existing cards are kept.

use std::path::{Path, PathBuf};

use ck_core::{CkError, CkResult, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::ast::{AstMap, Symbol, extract};
use crate::card::{
    AssetKind, Card, CatalogEntry, Determinism, Grade, LicenseClass, Runtime, SECTIONS,
};
use crate::hub::{Hub, LockEntry, SourceStatus};

const CODE_EXT: &[&str] = &["tsx", "jsx", "ts", "js"];
/// Longest doc text carried into a card (it is data, and cards stay small).
const DOC_CHARS: usize = 240;

pub mod caps {
    pub const INGEST: usize = 4 * 1024;
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IngestRequest {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IngestResult {
    pub source: String,
    pub assets: Vec<String>,
    /// Files without an exported component, or that did not parse cleanly.
    pub skipped: Vec<String>,
    /// Lines in catalog.jsonl after the rebuild.
    pub catalog: usize,
    #[serde(default, skip_serializing_if = "crate::sources::is_zero")]
    pub omitted: usize,
}

fn io(path: &Path, e: impl std::fmt::Display) -> CkError {
    CkError::new(ErrorCode::Storage, format!("{}: {e}", path.display()))
}

/// `GlowTitle` to `glow-title`.
pub fn kebab(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                out.push('-');
            }
            out.push(c.to_ascii_lowercase());
        } else if c.is_ascii_alphanumeric() {
            out.push(c);
        } else {
            out.push('-');
        }
    }
    out
}

fn files(dir: &Path) -> CkResult<Vec<PathBuf>> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).map_err(|e| io(&d, e))? {
            let p = e.map_err(|e| io(&d, e))?.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if p.is_dir() {
                if !matches!(name, "node_modules" | "dist" | "build" | ".git") {
                    stack.push(p);
                }
            } else {
                let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
                let skip = name.ends_with(".d.ts")
                    || name.contains(".test.")
                    || name.contains(".spec.")
                    || name.contains(".stories.");
                if CODE_EXT.contains(&ext) && !skip {
                    out.push(p);
                }
            }
        }
    }
    out.sort();
    Ok(out)
}

fn components(map: &AstMap) -> Vec<&Symbol> {
    map.outline
        .iter()
        .filter(|s| {
            s.exported
                && matches!(s.kind.as_str(), "const" | "function" | "default")
                && s.name
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_uppercase())
                && !s.name.ends_with("Props")
        })
        .collect()
}

fn cut(s: &str) -> String {
    let flat = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= DOC_CHARS {
        return flat;
    }
    let mut c: String = flat.chars().take(DOC_CHARS - 1).collect();
    c.push('…');
    c
}

struct Ctx<'a> {
    source: &'a str,
    lock: &'a LockEntry,
    license: &'a str,
}

fn card_for(ctx: &Ctx<'_>, rel: &str, map: &AstMap, sym: &Symbol) -> Card {
    let nondeterministic = ctx
        .lock
        .findings
        .iter()
        .any(|f| f.rule == "nondeterministic" && f.path.ends_with(rel));
    let frame_driven = map.primitives.iter().any(|p| p == "useCurrentFrame");
    let remotion = !map.primitives.is_empty();
    let determinism = match (frame_driven, nondeterministic) {
        (true, false) => Determinism::Native,
        _ => Determinism::Wrappable,
    };
    let runtime = match ctx.lock.license_class {
        LicenseClass::ReferenceOnly | LicenseClass::Unknown => Runtime::ReferenceOnly,
        _ if !remotion => Runtime::DeriveOnly,
        _ => Runtime::Direct,
    };
    let props = map
        .props
        .iter()
        .find(|p| p.name == format!("{}Props", sym.name))
        .or_else(|| map.props.first());
    let typed = props.is_some_and(|p| !p.fields.is_empty());
    // A component's own doc comment, else its props type's.
    let own_doc = if sym.doc.is_empty() {
        props
            .and_then(|p| map.outline.iter().find(|s| s.name == p.name))
            .map_or("", |s| s.doc.as_str())
    } else {
        sym.doc.as_str()
    };
    let grade = match (determinism, typed, own_doc.is_empty()) {
        (Determinism::Native, true, false) => Grade::A,
        (Determinism::Native, _, _) => Grade::B,
        _ => Grade::C,
    };
    let inputs = props.map_or_else(
        || "None declared.".to_owned(),
        |p| {
            p.fields
                .iter()
                .map(|f| {
                    format!(
                        "- {}{}: {}",
                        f.name,
                        if f.optional { "?" } else { "" },
                        if f.ty.is_empty() { "unknown" } else { &f.ty }
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        },
    );
    let doc = cut(own_doc);
    let what = if doc.is_empty() {
        format!(
            "Exported {} `{}` from `{rel}` (no doc comment).",
            sym.kind, sym.name
        )
    } else {
        doc.clone()
    };
    let use_when = if doc.is_empty() {
        format!(
            "- When a {} with these inputs fits the scene.",
            if remotion {
                "Remotion clip"
            } else {
                "React component"
            }
        )
    } else {
        format!(
            "- {}",
            doc.split(". ").next().unwrap_or(&doc).trim_end_matches('.')
        )
    };
    let mut avoid = Vec::new();
    if determinism == Determinism::Wrappable {
        avoid.push("- Frames may differ between renders: needs the frame-clock adapter and a determinism test first.".to_owned());
    }
    if ctx.lock.license_class != LicenseClass::Permissive {
        avoid.push(format!(
            "- License class `{}`: check the license mode.",
            ctx.lock.license_class.as_str()
        ));
    }
    if avoid.is_empty() {
        avoid.push("- When the look calls for different motion than its own.".into());
    }
    let parts = map
        .outline
        .iter()
        .map(|s| {
            format!(
                "- {} `{}` (lines {}-{})",
                s.kind, s.name, s.start_line, s.end_line
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let taste = if map.primitives.is_empty() {
        "No Remotion primitives; driven by its props.".to_owned()
    } else {
        format!("Animates with {}.", map.primitives.join(", "))
    };
    let section = |t: &str| -> String {
        match t {
            "What it is" => what.clone(),
            "Use when" => use_when.clone(),
            "Avoid when" => avoid.join("\n"),
            "Inputs" => inputs.clone(),
            "Safe to adapt" => {
                "Text, colors and timing through props; deeper changes through derive.".into()
            }
            "Structural parts" => parts.clone(),
            "Cannot do" => {
                "Not reviewed by a person yet; nothing beyond its declared inputs.".into()
            }
            "Taste notes" => taste.clone(),
            _ => String::new(),
        }
    };
    Card {
        id: format!("{}/{}", ctx.source, kebab(&sym.name)),
        name: sym.name.clone(),
        source: ctx.source.to_owned(),
        source_commit: ctx.lock.commit.clone(),
        kind: if remotion {
            AssetKind::RemotionClip
        } else {
            AssetKind::ReactComponent
        },
        runtime,
        determinism,
        license: ctx.license.to_owned(),
        spdx: ctx.lock.spdx.clone(),
        license_class: ctx.lock.license_class,
        grade,
        deps: map
            .imports
            .iter()
            .map(|i| i.from.clone())
            .filter(|f| !f.starts_with('.'))
            .collect(),
        cost: 2,
        duration: None,
        aspects: Vec::new(),
        tags: vec![
            kebab(&sym.name)
                .replace('-', " ")
                .split(' ')
                .map(str::to_owned)
                .collect::<Vec<_>>()
                .join("-"),
        ],
        collections: vec![ctx.source.to_owned()],
        reviewed: false,
        attribution: (ctx.lock.license_class == LicenseClass::Attribution)
            .then(|| format!("{} by {}, {}", sym.name, ctx.source, ctx.license)),
        sections: SECTIONS
            .iter()
            .map(|t| ((*t).to_owned(), section(t)))
            .collect(),
    }
}

fn write_ro(path: &Path, bytes: &[u8], read_only: bool) -> CkResult<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| io(dir, e))?;
    }
    if path.exists() {
        let mut p = std::fs::metadata(path)
            .map_err(|e| io(path, e))?
            .permissions();
        #[allow(clippy::permissions_set_readonly_false)]
        p.set_readonly(false);
        std::fs::set_permissions(path, p).map_err(|e| io(path, e))?;
    }
    std::fs::write(path, bytes).map_err(|e| io(path, e))?;
    if read_only {
        let mut p = std::fs::metadata(path)
            .map_err(|e| io(path, e))?
            .permissions();
        p.set_readonly(true);
        std::fs::set_permissions(path, p).map_err(|e| io(path, e))?;
    }
    Ok(())
}

/// Every card of every usable source, rebuilt into catalog.jsonl. Returns the line count.
pub fn rebuild_catalog(hub: &Hub) -> CkResult<usize> {
    let usable: Vec<String> = hub
        .sources()?
        .into_iter()
        .filter(|s| s.usable())
        .map(|s| s.id)
        .collect();
    let mut lines = Vec::new();
    for id in &usable {
        let dir = hub.assets().join(id);
        if !dir.is_dir() {
            continue;
        }
        let mut assets: Vec<PathBuf> = std::fs::read_dir(&dir)
            .map_err(|e| io(&dir, e))?
            .filter_map(|e| e.ok().map(|e| e.path().join("asset.md")))
            .filter(|p| p.is_file())
            .collect();
        assets.sort();
        for path in assets {
            let text = std::fs::read_to_string(&path).map_err(|e| io(&path, e))?;
            let card = Card::parse(&path, &text)?;
            let line = serde_json::to_string(&CatalogEntry::from(&card))
                .map_err(|e| CkError::new(ErrorCode::Server, e.to_string()))?;
            lines.push(line);
        }
    }
    let mut text = lines.join("\n");
    if !text.is_empty() {
        text.push('\n');
    }
    ck_core::storage::atomic_write(&hub.assets().join("catalog.jsonl"), text.as_bytes())?;
    Ok(lines.len())
}

pub fn ingest(hub: &Hub, req: &IngestRequest) -> CkResult<IngestResult> {
    let source = hub.source(&req.id)?;
    if source.status != SourceStatus::Vetted {
        return Err(CkError::new(
            ErrorCode::Request,
            format!("source {:?} is not vetted", source.id),
        )
        .with_hint("Run ck sources vet first; only vetted sources are ingested."));
    }
    let lock = hub.lock()?;
    let entry = lock.get(&source.id).ok_or_else(|| {
        CkError::new(
            ErrorCode::Request,
            format!("source {:?} has no lock entry", source.id),
        )
    })?;
    let quarantine = hub.quarantine().join(&source.id);
    let ctx = Ctx {
        source: &source.id,
        lock: entry,
        license: &source.license,
    };
    let out_dir = hub.assets().join(&source.id);
    let mut assets = Vec::new();
    let mut skipped = Vec::new();
    let mut index = Vec::new();
    for path in files(&quarantine)? {
        let rel = path
            .strip_prefix(&quarantine)
            .map_err(|e| io(&path, e))?
            .to_string_lossy()
            .replace('\\', "/");
        let text = std::fs::read_to_string(&path).map_err(|e| io(&path, e))?;
        let map = extract(&rel, &text);
        let comps = components(&map);
        if map.errors > 0 || comps.is_empty() {
            skipped.push(if map.errors > 0 {
                format!("{rel}: {} parse errors", map.errors)
            } else {
                format!("{rel}: no exported component")
            });
            continue;
        }
        for sym in comps {
            let card = card_for(&ctx, &rel, &map, sym);
            let name = card.id.rsplit('/').next().unwrap_or("").to_owned();
            let dir = out_dir.join(&name);
            let card_path = dir.join("asset.md");
            let owner = std::fs::read_to_string(&card_path)
                .ok()
                .and_then(|t| Card::parse(&card_path, &t).ok())
                .map(|c| c.section("Owner notes").to_owned())
                .filter(|n| !n.is_empty());
            let rendered = card.render(owner.as_deref());
            Card::parse(&card_path, &rendered)?;
            write_ro(&card_path, rendered.as_bytes(), false)?;
            let ast = serde_json::to_vec_pretty(&map)
                .map_err(|e| CkError::new(ErrorCode::Server, e.to_string()))?;
            write_ro(&dir.join("ast.json"), &ast, false)?;
            let file = path.file_name().map(PathBuf::from).unwrap_or_default();
            write_ro(&dir.join("src").join(file), text.as_bytes(), true)?;
            index.push(format!(
                "- {} ({}, grade {}): {}",
                card.id,
                card.kind.as_str(),
                card.grade.as_str(),
                CatalogEntry::from(&card).use_when
            ));
            assets.push(card.id);
        }
    }
    let source_md = format!(
        "# {}\n\n- License: {} (SPDX: {}, class: {})\n- Commit: {}\n- Vetted: {}\n- Runtime: \
         Remotion components run in the render path; others are derive-only.\n",
        source.id,
        source.license,
        if entry.spdx.is_empty() {
            "none"
        } else {
            &entry.spdx
        },
        entry.license_class.as_str(),
        entry.commit,
        entry.vetted_at
    );
    write_ro(&out_dir.join("SOURCE.md"), source_md.as_bytes(), false)?;
    let index_md = format!("# {} assets\n\n{}\n", source.id, index.join("\n"));
    write_ro(&out_dir.join("INDEX.md"), index_md.as_bytes(), false)?;
    let catalog = rebuild_catalog(hub)?;
    Ok(IngestResult {
        source: source.id,
        assets,
        skipped,
        catalog,
        omitted: 0,
    })
}

#[cfg(test)]
mod tests;

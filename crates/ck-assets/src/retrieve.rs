//! The retrieval ladder with its budget gate (spec 7.3, 9.2): search (top 5 compact lines),
//! card (at most 2 per decision), skeleton (about 600 tokens), slice (one symbol, about 800
//! tokens), full source (with a reason, once per decision). A decision lasts until the project
//! revision changes; the ledger is `.clipkit/retrieval.json`. Every answer reports the tokens
//! spent in the window against the 1,500 token target.

use std::path::{Path, PathBuf};

use ck_core::{CkError, CkResult, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{STORED, STRING, Schema, TEXT, Value};
use tantivy::{Index, TantivyDocument, doc};

use crate::ast::{AstMap, slice as slice_lines};
use crate::card::{AssetKind, CatalogEntry, Grade, LicenseMode};
use crate::hub::Hub;
use crate::sources::catalog;

/// Spec 9.2: asset retrieval per layering pass.
pub const TOKEN_TARGET: u64 = 1500;
pub const MAX_CARDS: usize = 2;
pub const MAX_SOURCES: usize = 1;
pub const DEFAULT_HITS: usize = 5;
/// Bytes per token used by the meter (ck's own output sizes; labeled as an estimate).
pub const BYTES_PER_TOKEN: u64 = 4;
pub const LEDGER_FILE: &str = "retrieval.json";

pub mod caps {
    pub const SEARCH: usize = 2 * 1024;
    pub const CARD: usize = 3 * 1024;
    /// About 600 tokens.
    pub const SKELETON: usize = 2400 + 256;
    /// About 800 tokens.
    pub const SLICE: usize = 3200 + 256;
    pub const SOURCE: usize = 16 * 1024;
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Ledger {
    rev: u64,
    cards: Vec<String>,
    sources: Vec<String>,
    tokens: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Budget {
    /// Tokens spent on retrieval in this decision (ck output bytes / 4, an estimate).
    pub used: u64,
    pub target: u64,
    pub cards: usize,
    pub sources: usize,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssetSearchRequest {
    #[serde(default)]
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<AssetKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// For example `9:16`; assets with no aspects listed match any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aspect: Option<String>,
    /// Default: the brief's licenseMode, else commercial-safe.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<LicenseMode>,
    /// Lowest grade accepted (A is best).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grade: Option<Grade>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collection: Option<String>,
    /// At most 5.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AssetSearchResult {
    pub mode: LicenseMode,
    /// `id kind grade licenseClass: use when` per hit.
    pub hits: Vec<String>,
    pub budget: Budget,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssetIdRequest {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssetSliceRequest {
    pub id: String,
    /// Symbol name from the skeleton's outline.
    pub symbol: String,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssetSourceRequest {
    pub id: String,
    /// Why the card, skeleton and slice are not enough (one line, at least 10 characters).
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AssetText {
    pub id: String,
    pub text: String,
    pub budget: Budget,
}

/// Where retrieval reads and records.
pub struct Retrieval {
    pub hub: Hub,
    /// The project's `.clipkit` folder (ledger and brief); absent outside a project.
    pub project: Option<PathBuf>,
}

fn io(path: &Path, e: impl std::fmt::Display) -> CkError {
    CkError::new(ErrorCode::Storage, format!("{}: {e}", path.display()))
}

fn budget_err(msg: String) -> CkError {
    CkError::new(ErrorCode::Budget, msg)
}

impl Retrieval {
    pub fn new(hub: Hub, project_dir: &Path) -> Self {
        let root = project_dir.join(".clipkit");
        Self {
            hub,
            project: root.join("project.json").is_file().then_some(root),
        }
    }

    fn project_json(&self) -> Option<serde_json::Value> {
        let p = self.project.as_ref()?.join("project.json");
        serde_json::from_str(&std::fs::read_to_string(p).ok()?).ok()
    }

    fn mode(&self, asked: Option<LicenseMode>) -> LicenseMode {
        asked
            .or_else(|| {
                let p = self.project_json()?;
                let m = p.get("brief")?.get("licenseMode")?.as_str()?.to_owned();
                LicenseMode::parse(&m)
            })
            .unwrap_or(LicenseMode::CommercialSafe)
    }

    fn ledger(&self) -> CkResult<Ledger> {
        let rev = self
            .project_json()
            .and_then(|p| p.get("rev")?.as_u64())
            .unwrap_or(0);
        let stored: Ledger = self
            .project
            .as_ref()
            .and_then(|r| std::fs::read_to_string(r.join(LEDGER_FILE)).ok())
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();
        // The ledger lives for one agent turn: handoff clears it (spec 7.3), revisions do not.
        Ok(Ledger { rev, ..stored })
    }

    fn save(&self, ledger: &Ledger) -> CkResult<()> {
        let Some(root) = &self.project else {
            return Ok(());
        };
        let text = serde_json::to_vec(ledger)
            .map_err(|e| CkError::new(ErrorCode::Server, e.to_string()))?;
        ck_core::storage::atomic_write(&root.join(LEDGER_FILE), &text)
    }

    /// Records `bytes` of output and returns the budget after it.
    fn spend(&self, ledger: &mut Ledger, bytes: usize) -> CkResult<Budget> {
        if ledger.tokens >= TOKEN_TARGET {
            return Err(CkError::new(
                ErrorCode::Budget,
                format!("retrieval budget of {TOKEN_TARGET} tokens is spent for this turn"),
            )
            .with_hint("Work from what you have, or hand off to start a new turn."));
        }
        ledger.tokens += (bytes as u64).div_ceil(BYTES_PER_TOKEN);
        self.save(ledger)?;
        Ok(Budget {
            used: ledger.tokens,
            target: TOKEN_TARGET,
            cards: ledger.cards.len(),
            sources: ledger.sources.len(),
        })
    }

    fn usable(&self) -> CkResult<Vec<CatalogEntry>> {
        let usable: Vec<String> = self
            .hub
            .sources()?
            .into_iter()
            .filter(|s| s.usable())
            .map(|s| s.id)
            .collect();
        Ok(catalog(&self.hub)?
            .into_iter()
            .filter(|c| usable.contains(&c.source))
            .collect())
    }

    /// The catalog entry for `id` when its source is usable; otherwise not found (so an
    /// unverified source never leaks through the ladder).
    fn entry(&self, id: &str) -> CkResult<CatalogEntry> {
        self.usable()?
            .into_iter()
            .find(|c| c.id == id)
            .ok_or_else(|| {
                CkError::new(ErrorCode::NotFound, format!("no asset {id:?}"))
                    .with_hint("Run ck assets search to find asset ids.")
            })
    }

    fn asset_dir(&self, id: &str) -> PathBuf {
        self.hub.assets().join(id)
    }

    pub fn search(&self, req: &AssetSearchRequest) -> CkResult<AssetSearchResult> {
        let mode = self.mode(req.mode);
        let limit = req.limit.unwrap_or(DEFAULT_HITS).clamp(1, DEFAULT_HITS);
        let entries: Vec<CatalogEntry> = self
            .usable()?
            .into_iter()
            .filter(|c| mode.allows(c.license_class))
            .filter(|c| req.kind.is_none_or(|k| c.kind == k))
            .filter(|c| req.tag.as_ref().is_none_or(|t| c.tags.contains(t)))
            .filter(|c| {
                req.aspect
                    .as_ref()
                    .is_none_or(|a| c.aspects.is_empty() || c.aspects.contains(a))
            })
            .filter(|c| req.grade.is_none_or(|g| c.grade <= g))
            .filter(|c| {
                req.collection
                    .as_ref()
                    .is_none_or(|k| c.collections.contains(k))
            })
            .collect();
        let ranked = rank(&entries, &req.query)?;
        let hits: Vec<String> = ranked
            .into_iter()
            .take(limit)
            .map(|c| {
                format!(
                    "{} {} {} {}: {}",
                    c.id,
                    c.kind.as_str(),
                    c.grade.as_str(),
                    c.license_class.as_str(),
                    c.use_when
                )
            })
            .collect();
        let mut ledger = self.ledger()?;
        let bytes = hits.iter().map(String::len).sum();
        let budget = self.spend(&mut ledger, bytes)?;
        Ok(AssetSearchResult { mode, hits, budget })
    }

    pub fn card(&self, req: &AssetIdRequest) -> CkResult<AssetText> {
        self.entry(&req.id)?;
        let mut ledger = self.ledger()?;
        if !ledger.cards.contains(&req.id) {
            if ledger.cards.len() >= MAX_CARDS {
                return Err(budget_err(format!(
                    "{MAX_CARDS} cards already read for this decision ({})",
                    ledger.cards.join(", ")
                )));
            }
            ledger.cards.push(req.id.clone());
        }
        let path = self.asset_dir(&req.id).join("asset.md");
        let text = std::fs::read_to_string(&path).map_err(|e| io(&path, e))?;
        let budget = self.spend(&mut ledger, text.len())?;
        Ok(AssetText {
            id: req.id.clone(),
            text,
            budget,
        })
    }

    fn ast(&self, id: &str) -> CkResult<AstMap> {
        let path = self.asset_dir(id).join("ast.json");
        let text = std::fs::read_to_string(&path).map_err(|e| io(&path, e))?;
        serde_json::from_str(&text).map_err(|e| io(&path, e))
    }

    pub fn skeleton(&self, req: &AssetIdRequest) -> CkResult<AssetText> {
        self.entry(&req.id)?;
        let map = self.ast(&req.id)?;
        let mut s = format!("file: {}\n", map.file);
        if !map.primitives.is_empty() {
            s.push_str(&format!("animates: {}\n", map.primitives.join(", ")));
        }
        for i in &map.imports {
            s.push_str(&format!("import {} from {}\n", i.names.join(", "), i.from));
        }
        for p in &map.props {
            s.push_str(&format!("props {}:\n", p.name));
            for f in &p.fields {
                let opt = if f.optional { "?" } else { "" };
                s.push_str(&format!("  {}{opt}: {}\n", f.name, f.ty));
            }
        }
        s.push_str("outline:\n");
        for o in &map.outline {
            let export = if o.exported { "export " } else { "" };
            s.push_str(&format!(
                "  {export}{} {} (lines {}-{})\n",
                o.kind, o.name, o.start_line, o.end_line
            ));
        }
        let text = cap_text(s, caps::SKELETON - 256);
        let mut ledger = self.ledger()?;
        let budget = self.spend(&mut ledger, text.len())?;
        Ok(AssetText {
            id: req.id.clone(),
            text,
            budget,
        })
    }

    fn source_text(&self, id: &str, map: &AstMap) -> CkResult<String> {
        let name = Path::new(&map.file)
            .file_name()
            .map(PathBuf::from)
            .unwrap_or_default();
        let path = self.asset_dir(id).join("src").join(name);
        std::fs::read_to_string(&path).map_err(|e| io(&path, e))
    }

    pub fn slice(&self, req: &AssetSliceRequest) -> CkResult<AssetText> {
        self.entry(&req.id)?;
        let map = self.ast(&req.id)?;
        let sym = map
            .outline
            .iter()
            .find(|o| o.name == req.symbol)
            .ok_or_else(|| {
                CkError::new(ErrorCode::NotFound, format!("no symbol {:?}", req.symbol))
                    .with_hint("Read the skeleton for the outline.")
            })?;
        let src = self.source_text(&req.id, &map)?;
        let text = cap_text(
            slice_lines(&src, sym.start_line, sym.end_line),
            caps::SLICE - 256,
        );
        let mut ledger = self.ledger()?;
        let budget = self.spend(&mut ledger, text.len())?;
        Ok(AssetText {
            id: req.id.clone(),
            text,
            budget,
        })
    }

    pub fn source(&self, req: &AssetSourceRequest) -> CkResult<AssetText> {
        self.entry(&req.id)?;
        if req.reason.trim().chars().count() < 10 {
            return Err(CkError::new(
                ErrorCode::Request,
                "full source needs a one-line reason (at least 10 characters)",
            ));
        }
        let mut ledger = self.ledger()?;
        if !ledger.sources.contains(&req.id) {
            if ledger.sources.len() >= MAX_SOURCES {
                return Err(budget_err(format!(
                    "full source already read for this decision ({})",
                    ledger.sources.join(", ")
                )));
            }
            ledger.sources.push(req.id.clone());
        }
        let map = self.ast(&req.id)?;
        let text = cap_text(self.source_text(&req.id, &map)?, caps::SOURCE - 256);
        let budget = self.spend(&mut ledger, text.len())?;
        Ok(AssetText {
            id: req.id.clone(),
            text,
            budget,
        })
    }
}

/// Cut `s` to at most `max` bytes on a line boundary, saying so.
fn cap_text(s: String, max: usize) -> String {
    if s.len() <= max {
        return s;
    }
    let mut out = String::new();
    for line in s.lines() {
        if out.len() + line.len() + 1 + 24 > max {
            break;
        }
        out.push_str(line);
        out.push('\n');
    }
    out.push_str("… (cut to the cap)\n");
    out
}

/// BM25 over name, tags, kind and use-when (tantivy, in memory), then grade; an empty query
/// keeps catalog order by grade.
fn rank<'a>(entries: &'a [CatalogEntry], query: &str) -> CkResult<Vec<&'a CatalogEntry>> {
    let err = |e: tantivy::TantivyError| CkError::new(ErrorCode::Server, format!("search: {e}"));
    if query.trim().is_empty() {
        let mut all: Vec<&CatalogEntry> = entries.iter().collect();
        all.sort_by(|a, b| a.grade.cmp(&b.grade).then(a.id.cmp(&b.id)));
        return Ok(all);
    }
    let mut sb = Schema::builder();
    let pos = sb.add_u64_field("pos", STORED);
    let text = sb.add_text_field("text", TEXT);
    let id = sb.add_text_field("id", STRING);
    let index = Index::create_in_ram(sb.build());
    let mut w = index
        .writer_with_num_threads::<TantivyDocument>(1, 15_000_000)
        .map_err(err)?;
    for (n, e) in entries.iter().enumerate() {
        let body = format!(
            "{} {} {} {} {} {}",
            e.name,
            e.tags.join(" "),
            e.kind.as_str().replace('-', " "),
            e.use_when,
            e.source,
            e.id.replace(['/', '-'], " ")
        );
        w.add_document(doc!(pos => n as u64, text => body, id => e.id.clone()))
            .map_err(err)?;
    }
    w.commit().map_err(err)?;
    let reader = index.reader().map_err(err)?;
    let searcher = reader.searcher();
    let (q, _) = QueryParser::for_index(&index, vec![text]).parse_query_lenient(query);
    let top = searcher
        .search(
            &q,
            &TopDocs::with_limit(entries.len().max(1)).order_by_score(),
        )
        .map_err(err)?;
    let mut hits: Vec<(f32, &CatalogEntry)> = Vec::new();
    for (score, addr) in top {
        let d: TantivyDocument = searcher.doc(addr).map_err(err)?;
        if let Some(n) = d.get_first(pos).and_then(|v| v.as_u64())
            && let Some(e) = entries.get(n as usize)
        {
            hits.push((score, e));
        }
    }
    hits.sort_by(|a, b| {
        b.0.total_cmp(&a.0)
            .then(a.1.grade.cmp(&b.1.grade))
            .then(a.1.id.cmp(&b.1.id))
    });
    Ok(hits.into_iter().map(|(_, e)| e).collect())
}

#[cfg(test)]
mod tests;

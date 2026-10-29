//! `ck sources list|add|gaps` (spec 6.2, 7.6): read and extend the source registry. Adding a
//! source records it unverified, unknown and disabled; only `sources vet` can change that.

use ck_core::{CkError, CkResult, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::card::{AssetKind, CatalogEntry, LicenseClass};
use crate::hub::{Hub, SourceEntry, SourceStatus};

pub mod caps {
    pub const SOURCES_LIST: usize = 6 * 1024;
    pub const SOURCES_WRITE: usize = 1024;
    pub const SOURCES_GAPS: usize = 2 * 1024;
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourcesListRequest {
    /// Only sources with this status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<SourceStatus>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SourcesList {
    /// `id status enabled|disabled licenseClass kind repo` per source.
    pub lines: Vec<String>,
    #[serde(default, skip_serializing_if = "crate::sources::is_zero")]
    pub omitted: usize,
}

pub(crate) fn is_zero(n: &usize) -> bool {
    *n == 0
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourcesAddRequest {
    /// Lowercase id: letters, digits and dashes.
    pub id: String,
    #[serde(default)]
    pub name: String,
    pub kind: String,
    /// `owner/name` on GitHub, when known (never guessed).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    /// The license the listing claims (recorded as a claim, not a finding).
    #[serde(default)]
    pub license: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SourceWriteResult {
    pub id: String,
    pub status: SourceStatus,
    pub enabled: bool,
    pub license_class: LicenseClass,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourcesGapsRequest {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SourcesGaps {
    pub sources: usize,
    pub usable: usize,
    pub unverified: usize,
    pub rejected: usize,
    /// Unverified sources with no repo yet (resolve from the official listing).
    pub without_repo: Vec<String>,
    /// Asset kinds with no searchable card.
    pub kinds_missing: Vec<AssetKind>,
    /// Searchable cards per kind.
    pub cards_by_kind: Vec<(AssetKind, usize)>,
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !id.starts_with('-')
}

fn valid_repo(repo: &str) -> bool {
    let mut parts = repo.split('/');
    let ok = |p: Option<&str>| {
        p.is_some_and(|p| {
            !p.is_empty()
                && !p.starts_with('.')
                && p.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        })
    };
    ok(parts.next()) && ok(parts.next()) && parts.next().is_none()
}

/// Reads `assets/catalog.jsonl` (missing means empty).
pub fn catalog(hub: &Hub) -> CkResult<Vec<CatalogEntry>> {
    let path = hub.assets().join("catalog.jsonl");
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(&path)
        .map_err(|e| CkError::new(ErrorCode::Storage, format!("{}: {e}", path.display())))?;
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            serde_json::from_str(l)
                .map_err(|e| CkError::new(ErrorCode::Asset, format!("{}: {e}", path.display())))
        })
        .collect()
}

pub fn sources_list(hub: &Hub, req: &SourcesListRequest) -> CkResult<SourcesList> {
    let lines = hub
        .sources()?
        .iter()
        .filter(|s| req.status.is_none_or(|st| st == s.status))
        .map(|s| {
            format!(
                "{} {} {} {} {} {}",
                s.id,
                serde_json::to_value(s.status)
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_owned))
                    .unwrap_or_default(),
                if s.enabled { "enabled" } else { "disabled" },
                s.license_class.as_str(),
                if s.kind.is_empty() { "-" } else { &s.kind },
                s.repo.as_deref().unwrap_or("-"),
            )
        })
        .collect();
    Ok(SourcesList { lines, omitted: 0 })
}

pub fn sources_add(hub: &Hub, req: &SourcesAddRequest) -> CkResult<SourceWriteResult> {
    if !valid_id(&req.id) {
        return Err(CkError::new(
            ErrorCode::Request,
            format!(
                "source id {:?} must be lowercase letters, digits and dashes",
                req.id
            ),
        ));
    }
    if let Some(r) = &req.repo
        && !valid_repo(r)
    {
        return Err(CkError::new(
            ErrorCode::Request,
            format!("repo {r:?} must be owner/name"),
        ));
    }
    let mut all = hub.sources()?;
    if all.iter().any(|s| s.id == req.id) {
        return Err(CkError::new(
            ErrorCode::Request,
            format!("source {:?} already exists", req.id),
        ));
    }
    let entry = SourceEntry {
        id: req.id.clone(),
        name: if req.name.is_empty() {
            req.id.clone()
        } else {
            req.name.clone()
        },
        kind: req.kind.clone(),
        license: if req.license.is_empty() {
            "Unverified".into()
        } else {
            format!("Unverified (claimed: {})", req.license)
        },
        license_class: LicenseClass::Unknown,
        status: SourceStatus::Unverified,
        enabled: false,
        repo: req.repo.clone(),
        extra: Default::default(),
    };
    all.push(entry);
    hub.save_sources(&all)?;
    Ok(SourceWriteResult {
        id: req.id.clone(),
        status: SourceStatus::Unverified,
        enabled: false,
        license_class: LicenseClass::Unknown,
    })
}

pub fn sources_gaps(hub: &Hub, _req: &SourcesGapsRequest) -> CkResult<SourcesGaps> {
    let sources = hub.sources()?;
    let usable: Vec<&str> = sources
        .iter()
        .filter(|s| s.usable())
        .map(|s| s.id.as_str())
        .collect();
    let cards: Vec<CatalogEntry> = catalog(hub)?
        .into_iter()
        .filter(|c| usable.contains(&c.source.as_str()))
        .collect();
    let by_kind: Vec<(AssetKind, usize)> = AssetKind::ALL
        .iter()
        .map(|k| (*k, cards.iter().filter(|c| c.kind == *k).count()))
        .collect();
    Ok(SourcesGaps {
        sources: sources.len(),
        usable: usable.len(),
        unverified: sources
            .iter()
            .filter(|s| s.status == SourceStatus::Unverified)
            .count(),
        rejected: sources
            .iter()
            .filter(|s| s.status == SourceStatus::Rejected)
            .count(),
        without_repo: sources
            .iter()
            .filter(|s| s.status == SourceStatus::Unverified && s.repo.is_none())
            .map(|s| s.id.clone())
            .collect(),
        kinds_missing: by_kind
            .iter()
            .filter(|(_, n)| *n == 0)
            .map(|(k, _)| *k)
            .collect(),
        cards_by_kind: by_kind.into_iter().filter(|(_, n)| *n > 0).collect(),
    })
}

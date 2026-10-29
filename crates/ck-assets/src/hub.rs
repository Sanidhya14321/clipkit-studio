//! Source Hub (spec 7.6): the registry of external sources (`sources.yaml`, JSON is valid
//! YAML), their pinned vetting results (`sources.lock`) and the ingested knowledge base under
//! `assets/`. Only vetted and enabled sources are searchable.
//!
//! The hub is the folder holding `sources.yaml`: `CK_HUB` when set, else the nearest folder at
//! or above the start folder that has one.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ck_core::{CkError, CkResult, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::card::LicenseClass;

pub const SOURCES_FILE: &str = "sources.yaml";
pub const LOCK_FILE: &str = "sources.lock";
/// Environment variable naming the hub folder.
pub const HUB_ENV: &str = "CK_HUB";

/// Vetting status of a source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum SourceStatus {
    Unverified,
    /// Vetting ran and passed; the license was read from the source.
    Vetted,
    /// Vetting ran and found a blocker.
    Rejected,
}

/// One `sources.yaml` entry. Fields this version does not know are kept as they are.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SourceEntry {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub kind: String,
    /// The license as recorded (claimed until vetted).
    #[serde(default)]
    pub license: String,
    pub license_class: LicenseClass,
    pub status: SourceStatus,
    #[serde(default)]
    pub enabled: bool,
    /// `owner/name` on GitHub, or null when unresolved (never guessed).
    #[serde(default)]
    pub repo: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl SourceEntry {
    /// Searchable: vetted and enabled (spec 14.1).
    pub fn usable(&self) -> bool {
        self.status == SourceStatus::Vetted && self.enabled
    }
}

/// Pinned vetting result for one source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LockEntry {
    /// Git commit, or the content hash of a local copy.
    pub commit: String,
    /// SHA-256 of the license text as read.
    pub license_hash: String,
    pub license_class: LicenseClass,
    /// SPDX id when the text matches one; empty for custom terms.
    pub spdx: String,
    pub vetted_at: String,
    pub findings: Vec<Finding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    /// `blocker` stops ingestion; `warning` is reported.
    pub severity: Severity,
    pub rule: String,
    pub path: String,
    pub detail: String,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    Warning,
    Blocker,
}

#[derive(Debug, Clone)]
pub struct Hub {
    pub root: PathBuf,
}

fn io(path: &Path, e: impl std::fmt::Display) -> CkError {
    CkError::new(ErrorCode::Storage, format!("{}: {e}", path.display()))
}

impl Hub {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// `CK_HUB`, else the nearest folder at or above `start` with a `sources.yaml`.
    pub fn locate(start: &Path) -> CkResult<Self> {
        if let Some(dir) = std::env::var_os(HUB_ENV).filter(|v| !v.is_empty()) {
            return Ok(Self::new(dir));
        }
        let start = std::path::absolute(start).map_err(|e| io(start, e))?;
        start
            .ancestors()
            .find(|d| d.join(SOURCES_FILE).is_file())
            .map(Self::new)
            .ok_or_else(|| {
                CkError::new(ErrorCode::NotFound, "no sources.yaml found")
                    .with_hint("Set CK_HUB to the folder holding sources.yaml.")
            })
    }

    pub fn assets(&self) -> PathBuf {
        self.root.join("assets")
    }

    pub fn quarantine(&self) -> PathBuf {
        self.assets().join("quarantine")
    }

    pub fn sources(&self) -> CkResult<Vec<SourceEntry>> {
        let path = self.root.join(SOURCES_FILE);
        let text = std::fs::read_to_string(&path).map_err(|e| io(&path, e))?;
        serde_json::from_str(&text)
            .map_err(|e| CkError::new(ErrorCode::Asset, format!("{}: {e}", path.display())))
    }

    pub fn save_sources(&self, sources: &[SourceEntry]) -> CkResult<()> {
        let mut text = serde_json::to_string_pretty(sources)
            .map_err(|e| CkError::new(ErrorCode::Server, e.to_string()))?;
        text.push('\n');
        ck_core::storage::atomic_write(&self.root.join(SOURCES_FILE), text.as_bytes())
    }

    pub fn lock(&self) -> CkResult<BTreeMap<String, LockEntry>> {
        let path = self.root.join(LOCK_FILE);
        if !path.is_file() {
            return Ok(BTreeMap::new());
        }
        let text = std::fs::read_to_string(&path).map_err(|e| io(&path, e))?;
        serde_json::from_str(&text)
            .map_err(|e| CkError::new(ErrorCode::Asset, format!("{}: {e}", path.display())))
    }

    pub fn save_lock(&self, lock: &BTreeMap<String, LockEntry>) -> CkResult<()> {
        let mut text = serde_json::to_string_pretty(lock)
            .map_err(|e| CkError::new(ErrorCode::Server, e.to_string()))?;
        text.push('\n');
        ck_core::storage::atomic_write(&self.root.join(LOCK_FILE), text.as_bytes())
    }

    pub fn source(&self, id: &str) -> CkResult<SourceEntry> {
        self.sources()?
            .into_iter()
            .find(|s| s.id == id)
            .ok_or_else(|| CkError::new(ErrorCode::NotFound, format!("no source {id:?}")))
    }

    /// Replace one entry, keeping the file's order.
    pub fn put_source(&self, entry: SourceEntry) -> CkResult<()> {
        let mut all = self.sources()?;
        match all.iter_mut().find(|s| s.id == entry.id) {
            Some(s) => *s = entry,
            None => all.push(entry),
        }
        self.save_sources(&all)
    }
}

#[cfg(test)]
mod tests;

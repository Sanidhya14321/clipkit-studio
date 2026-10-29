//! `ck license report` (spec 7.6, 14.1): every asset the project uses, with its license, class,
//! attribution line and notes, checked against the brief's license mode. ClipKit's own starter
//! content (recipes, core clips, looks) is listed as original; the Remotion runtime license is
//! reported separately from every asset's license.

use std::collections::BTreeSet;
use std::path::Path;

use ck_core::{CkError, CkResult, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::card::{Card, LicenseClass, LicenseMode};
use crate::hub::Hub;

pub mod caps {
    pub const REPORT: usize = 8 * 1024;
}

/// The runtime line, kept apart from asset licenses (spec 14.1). It points at the license
/// rather than restating its terms.
pub const REMOTION_RUNTIME: &str = "Remotion (the rendering runtime) is under the Remotion License; read remotion.dev/license \
     to see whether your use needs a company license.";

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LicenseReportRequest {
    /// Check against this mode instead of the brief's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<LicenseMode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AssetLicense {
    pub id: String,
    pub license: String,
    pub spdx: String,
    pub license_class: LicenseClass,
    pub allowed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attribution: Option<String>,
    /// Source notes (custom license, trademark, runtime).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LicenseReport {
    pub mode: LicenseMode,
    /// True when every asset is allowed in the mode.
    pub ok: bool,
    pub assets: Vec<AssetLicense>,
    /// ClipKit's own content used (recipes, core clips): original work.
    pub original: Vec<String>,
    /// Assets the report could not resolve (not ingested, or source not usable).
    pub unresolved: Vec<String>,
    pub runtime: String,
    #[serde(default, skip_serializing_if = "crate::sources::is_zero")]
    pub omitted: usize,
}

fn io(path: &Path, e: impl std::fmt::Display) -> CkError {
    CkError::new(ErrorCode::Storage, format!("{}: {e}", path.display()))
}

/// Builds the report for the project stored under `clipkit` (its `.clipkit` folder).
pub fn license_report(
    hub: &Hub,
    clipkit: &Path,
    req: &LicenseReportRequest,
) -> CkResult<LicenseReport> {
    let doc = ck_core::storage::Store::new(clipkit).load()?;
    let project = doc.project();
    let mode = req
        .mode
        .or_else(|| {
            project
                .brief
                .license_mode
                .as_deref()
                .and_then(LicenseMode::parse)
        })
        .unwrap_or(LicenseMode::CommercialSafe);
    let mut used: BTreeSet<String> = project.assets_used.iter().cloned().collect();
    used.extend(
        project
            .items()
            .filter_map(|i| i.provenance.as_ref().map(|p| p.source_asset.clone())),
    );
    let sources = hub.sources().unwrap_or_default();
    let mut assets = Vec::new();
    let mut original = Vec::new();
    let mut unresolved = Vec::new();
    for id in used {
        if id.starts_with("recipe:") || id.starts_with("clip:") || !id.contains('/') {
            original.push(id);
            continue;
        }
        let path = hub.assets().join(&id).join("asset.md");
        let source = sources
            .iter()
            .find(|s| id.starts_with(&format!("{}/", s.id)));
        let card = match (std::fs::read_to_string(&path), source) {
            (Ok(text), Some(s)) if s.usable() => Card::parse(&path, &text)?,
            (Err(e), _) if e.kind() != std::io::ErrorKind::NotFound => return Err(io(&path, e)),
            _ => {
                unresolved.push(id);
                continue;
            }
        };
        let note = source
            .and_then(|s| s.extra.get("licenseNote"))
            .and_then(|v| v.as_str())
            .map(str::to_owned);
        assets.push(AssetLicense {
            allowed: mode.allows(card.license_class),
            id,
            license: card.license,
            spdx: card.spdx,
            license_class: card.license_class,
            attribution: card.attribution,
            note,
        });
    }
    Ok(LicenseReport {
        mode,
        ok: assets.iter().all(|a| a.allowed) && unresolved.is_empty(),
        assets,
        original,
        unresolved,
        runtime: REMOTION_RUNTIME.to_owned(),
        omitted: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::tests::ingested_hub;

    fn project(assets: &[&str], mode: Option<&str>) -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        let mut p = ck_core::Project::new("p1", "Demo", ck_core::model::Mode::Generate);
        p.assets_used = assets.iter().map(|a| (*a).to_owned()).collect();
        p.brief.license_mode = mode.map(str::to_owned);
        ck_core::storage::Store::new(d.path()).init(&p).unwrap();
        d
    }

    #[test]
    fn the_report_lists_licenses_and_blocks_restricted_assets_in_commercial_safe_mode() {
        let (_h, hub) = ingested_hub();
        let p = project(
            &[
                "recipe:hook-title",
                "pack-mit/glow-title",
                "pack-restricted/news-ticker",
                "gone/x",
            ],
            None,
        );
        let r = license_report(&hub, p.path(), &LicenseReportRequest::default()).unwrap();
        assert_eq!(r.mode, LicenseMode::CommercialSafe);
        assert!(!r.ok);
        assert_eq!(r.original, ["recipe:hook-title"]);
        assert_eq!(r.unresolved, ["gone/x"]);
        let by: Vec<(&str, bool)> = r
            .assets
            .iter()
            .map(|a| (a.id.as_str(), a.allowed))
            .collect();
        assert_eq!(
            by,
            [
                ("pack-mit/glow-title", true),
                ("pack-restricted/news-ticker", false)
            ]
        );
        assert_eq!(r.assets[0].spdx, "MIT");
        assert!(r.runtime.contains("Remotion License"));

        let personal = license_report(
            &hub,
            p.path(),
            &LicenseReportRequest {
                mode: Some(LicenseMode::Personal),
            },
        )
        .unwrap();
        assert!(personal.assets.iter().all(|a| a.allowed));
    }
}

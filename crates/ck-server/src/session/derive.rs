//! `assets derive` (spec 7.4): the codemod copy is written under `.clipkit/derived/<name>/` by
//! ck-assets; this records the asset in the project's `assetsUsed` as one agent revision, so
//! the license report lists it.

use ck_assets::card::LicenseMode;
use ck_assets::hub::Hub;
use ck_core::CkResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::Session;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeriveRequest {
    /// Asset id from `assets search`.
    pub id: String,
    /// Folder name for the copy (lowercase, digits, dashes).
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeriveResult {
    pub rev: u64,
    pub name: String,
    pub files: Vec<String>,
    pub adaptations: Vec<String>,
    #[serde(default, skip_serializing_if = "crate::session::is_zero")]
    pub omitted: usize,
}

impl Session {
    pub fn derive(&self, req: &DeriveRequest) -> CkResult<DeriveResult> {
        let workdir = self.workdir();
        let hub = Hub::locate(&workdir)?;
        let mode = {
            let doc = self.doc()?;
            doc.project()
                .brief
                .license_mode
                .as_deref()
                .and_then(LicenseMode::parse)
                .unwrap_or(LicenseMode::CommercialSafe)
        };
        let d = ck_assets::derive::derive(&hub, &workdir, &req.id, &req.name, mode)?;
        let mut doc = self.doc()?;
        let p = doc.project();
        let rev = if p.assets_used.contains(&req.id) {
            p.rev
        } else {
            let ops = json!([{ "op": "add", "path": "/assetsUsed/-", "value": req.id }]);
            self.commit_agent(&mut doc, ops, format!("derive: {} as {}", req.id, req.name))?
                .rev
        };
        Ok(DeriveResult {
            rev,
            name: d.name,
            files: d.files,
            adaptations: d.provenance.adaptations,
            omitted: 0,
        })
    }
}

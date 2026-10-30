//! `assets derive` (spec 7.4): the codemod copy is written under `.clipkit/derived/<name>/` by
//! ck-assets; this records the asset in the project's `assetsUsed` as one agent revision, so
//! the license report lists it.

use ck_assets::card::LicenseMode;
use ck_assets::hub::Hub;
use ck_core::{CkError, CkResult, ErrorCode};
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_adapter: Option<String>,
    #[serde(default, skip_serializing_if = "crate::session::is_zero")]
    pub omitted: usize,
}

impl Session {
    pub fn derive(&self, req: &DeriveRequest) -> CkResult<DeriveResult> {
        let workdir = self.workdir();
        let hub = Hub::locate(&workdir)?;
        // Hold the project lock while preparing a new name. Existing copies
        // are immutable: re-derive under a new name, never overwrite edits.
        let mut doc = self.doc()?;
        let p = doc.project();
        let mode = p
            .brief
            .license_mode
            .as_deref()
            .and_then(LicenseMode::parse)
            .unwrap_or(LicenseMode::CommercialSafe);
        if req.name.is_empty()
            || !req
                .name
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            return Err(CkError::new(
                ErrorCode::Request,
                "derived name must be lowercase letters, digits and dashes",
            ));
        }
        let target = workdir.join(".clipkit/derived").join(&req.name);
        if target.exists()
            || p.derived
                .iter()
                .any(|e| e["id"].as_str() == Some(&req.name))
        {
            return Err(CkError::new(ErrorCode::Request,"derived name already exists").with_hint("Choose a new name; existing derived files and registered hashes are immutable."));
        }
        let d = ck_assets::derive::derive(&hub, &workdir, &req.id, &req.name, mode)?;
        let native = ck_assets::native_adapter::adapter_for(&req.id);
        let mut ops = Vec::new();
        if !p.assets_used.contains(&req.id) {
            ops.push(json!({"op":"add","path":"/assetsUsed/-","value":req.id}));
        }
        if let Some((adapter, _)) = native {
            ops.push(json!({"op":"add","path":"/derived/-","value":{"id":req.name,"sourceAsset":req.id,"nativeAdapter":adapter,"mode":"native-reimplementation","sourceHash":d.provenance.hash}}));
        }

        let rev = if ops.is_empty() {
            p.rev
        } else {
            let ops = serde_json::Value::Array(ops);
            match self.commit_agent(&mut doc, ops, format!("derive: {} as {}", req.id, req.name)) {
                Ok(rev) => rev.rev,
                Err(error) => {
                    let _ = std::fs::remove_dir_all(&target);
                    return Err(error);
                }
            }
        };
        Ok(DeriveResult {
            rev,
            name: d.name,
            files: d.files,
            adaptations: d.provenance.adaptations,
            native_adapter: native.map(|(adapter, _)| adapter.to_owned()),
            omitted: 0,
        })
    }
}

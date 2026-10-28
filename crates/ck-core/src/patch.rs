//! The patch engine: RFC 6902 patches with revision and schema gates (spec 4.2).
//!
//! Diffs and inverses are computed on the *content* of a project: the project with engine-owned
//! fields (rev, item revs, locks, pipeline, agent) removed. Applying a content diff never touches
//! those fields, so undo and redo can reuse the same path as any other patch.

use std::collections::{BTreeMap, BTreeSet};

use json_patch::Patch;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{CkError, CkResult, ErrorCode};
use crate::model::{Author, Project};

/// Top-level fields only the engine may change.
const PROTECTED: &[&str] = &["/rev", "/schemaVersion", "/locks", "/pipeline", "/agent"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PatchRequest {
    /// The project revision the patch was built against.
    pub base_rev: u64,
    /// Optional per-layer revisions the caller read; any mismatch makes the patch stale.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub layer_revs: BTreeMap<String, u64>,
    pub ops: Patch,
    pub author: Author,
    #[serde(default)]
    pub label: String,
}

/// The result of applying a patch, before it is recorded in history.
#[derive(Debug, Clone, PartialEq)]
pub struct Applied {
    pub project: Project,
    /// Content diff from the old project to the new one.
    pub forward: Patch,
    /// Content diff that restores the old project.
    pub inverse: Patch,
    /// Ids of items added, removed or changed, sorted.
    pub touched: Vec<String>,
}

/// Apply `req` to `project` and return the new project at `project.rev + 1`.
/// Does not check locks; the caller (`Document`) applies the lock gate on `touched`.
pub fn apply(project: &Project, req: &PatchRequest) -> CkResult<Applied> {
    check_fresh(project, req)?;
    check_protected(&req.ops)?;

    let before = to_value(project)?;
    let mut doc = before.clone();
    json_patch::patch(&mut doc, &req.ops)
        .map_err(|e| CkError::new(ErrorCode::PatchInvalid, format!("patch failed: {e}")))?;
    for field in PROTECTED {
        if doc.pointer(field) != before.pointer(field) {
            return Err(CkError::new(
                ErrorCode::PatchInvalid,
                format!("{field} is read-only"),
            ));
        }
    }
    let mut next: Project = serde_json::from_value(doc).map_err(|e| {
        CkError::new(
            ErrorCode::PatchInvalid,
            format!("patched project does not parse: {e}"),
        )
    })?;
    next.validate()?;

    let (old_content, new_content) = (content(&before), content(&to_value(&next)?));
    if old_content == new_content {
        return Err(CkError::new(
            ErrorCode::PatchInvalid,
            "patch changes nothing",
        ));
    }
    let touched = touched_items(project, &next)?;

    // Engine bookkeeping: bump the project rev and stamp touched items; keep other items' revs.
    let old_revs: BTreeMap<&str, u64> = project.items().map(|i| (i.id.as_str(), i.rev)).collect();
    next.rev = project.rev + 1;
    for track in &mut next.tracks {
        for item in &mut track.items {
            item.rev = if touched.binary_search(&item.id).is_ok() {
                next.rev
            } else {
                old_revs.get(item.id.as_str()).copied().unwrap_or(next.rev)
            };
        }
    }

    Ok(Applied {
        forward: json_patch::diff(&old_content, &new_content),
        inverse: json_patch::diff(&new_content, &old_content),
        project: next,
        touched,
    })
}

/// The revision gate: refuse patches built on an older revision, naming only the changed layers.
fn check_fresh(project: &Project, req: &PatchRequest) -> CkResult<()> {
    if req.base_rev > project.rev {
        return Err(CkError::new(
            ErrorCode::PatchStale,
            format!(
                "baseRev {} is ahead of the project rev {}",
                req.base_rev, project.rev
            ),
        ));
    }
    let mut changed: BTreeSet<String> = BTreeSet::new();
    if req.base_rev < project.rev {
        changed.extend(
            project
                .items()
                .filter(|i| i.rev > req.base_rev)
                .map(|i| i.id.clone()),
        );
    }
    for (id, rev) in &req.layer_revs {
        if project.item(id).is_none_or(|i| i.rev != *rev) {
            changed.insert(id.clone());
        }
    }
    if req.base_rev < project.rev || !changed.is_empty() {
        return Err(CkError::new(
            ErrorCode::PatchStale,
            format!(
                "patch built on rev {}, project is at rev {}",
                req.base_rev, project.rev
            ),
        )
        .with_layers(changed));
    }
    Ok(())
}

fn check_protected(ops: &Patch) -> CkResult<()> {
    for op in ops.iter() {
        let v = serde_json::to_value(op)
            .map_err(|e| CkError::new(ErrorCode::PatchInvalid, e.to_string()))?;
        for key in ["path", "from"] {
            let Some(path) = v.get(key).and_then(Value::as_str) else {
                continue;
            };
            let hit = path.is_empty()
                || PROTECTED
                    .iter()
                    .any(|p| path == *p || path.starts_with(&format!("{p}/")));
            if hit {
                return Err(CkError::new(
                    ErrorCode::PatchInvalid,
                    format!("path {path:?} is read-only"),
                ));
            }
        }
    }
    Ok(())
}

fn to_value(project: &Project) -> CkResult<Value> {
    serde_json::to_value(project).map_err(|e| CkError::new(ErrorCode::Schema, e.to_string()))
}

/// The project without engine-owned fields. Removing keys never shifts array indices,
/// so content diffs apply cleanly to full project values.
pub(crate) fn content(full: &Value) -> Value {
    let mut v = full.clone();
    if let Some(obj) = v.as_object_mut() {
        for key in ["rev", "locks", "pipeline", "agent"] {
            obj.remove(key);
        }
    }
    if let Some(tracks) = v.get_mut("tracks").and_then(Value::as_array_mut) {
        for track in tracks {
            if let Some(items) = track.get_mut("items").and_then(Value::as_array_mut) {
                for item in items.iter_mut().filter_map(Value::as_object_mut) {
                    item.remove("rev");
                }
            }
        }
    }
    v
}

/// Ids of items whose content differs between `a` and `b` (added, removed or changed).
fn touched_items(a: &Project, b: &Project) -> CkResult<Vec<String>> {
    let index = |p: &Project| -> CkResult<BTreeMap<String, Value>> {
        p.items()
            .map(|i| {
                let mut v = serde_json::to_value(i)
                    .map_err(|e| CkError::new(ErrorCode::Schema, e.to_string()))?;
                if let Some(o) = v.as_object_mut() {
                    o.remove("rev");
                }
                Ok((i.id.clone(), v))
            })
            .collect()
    };
    let (ia, ib) = (index(a)?, index(b)?);
    let ids: BTreeSet<&String> = ia.keys().chain(ib.keys()).collect();
    Ok(ids
        .into_iter()
        .filter(|id| ia.get(*id) != ib.get(*id))
        .cloned()
        .collect())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::model::tests::sample;

    fn req(base_rev: u64, ops: Value) -> PatchRequest {
        PatchRequest {
            base_rev,
            layer_revs: BTreeMap::new(),
            ops: serde_json::from_value(ops).unwrap(),
            author: Author::Human,
            label: "test".into(),
        }
    }

    #[test]
    fn apply_bumps_revs_of_touched_items_only() {
        let p = sample();
        let out = apply(
            &p,
            &req(
                0,
                json!([{"op": "replace", "path": "/tracks/1/items/0/at", "value": 1.5}]),
            ),
        )
        .unwrap();
        assert_eq!(out.project.rev, 1);
        assert_eq!(out.touched, vec!["b".to_owned()]);
        assert_eq!(out.project.item("b").unwrap().rev, 1);
        assert_eq!(out.project.item("a").unwrap().rev, 0);
        // The inverse restores the content.
        let mut back = to_value(&out.project).unwrap();
        json_patch::patch(&mut back, &out.inverse).unwrap();
        assert_eq!(content(&back), content(&to_value(&p).unwrap()));
    }

    #[test]
    fn stale_patch_lists_only_changed_layers() {
        let p = sample();
        let p1 = apply(
            &p,
            &req(
                0,
                json!([{"op": "replace", "path": "/tracks/1/items/0/name", "value": "x"}]),
            ),
        )
        .unwrap()
        .project;
        let err = apply(
            &p1,
            &req(0, json!([{"op": "replace", "path": "/name", "value": "y"}])),
        )
        .unwrap_err();
        assert_eq!(err.code, ErrorCode::PatchStale);
        assert_eq!(err.layer_ids, vec!["b".to_owned()]);
        let err = apply(&p1, &req(5, json!([]))).unwrap_err();
        assert_eq!(err.code, ErrorCode::PatchStale);
        let mut r = req(1, json!([{"op": "replace", "path": "/name", "value": "y"}]));
        r.layer_revs.insert("a".into(), 7);
        assert_eq!(apply(&p1, &r).unwrap_err().layer_ids, vec!["a".to_owned()]);
    }

    #[test]
    fn protected_fields_and_bad_patches_are_refused() {
        let p = sample();
        for (ops, code) in [
            (
                json!([{"op": "replace", "path": "/rev", "value": 9}]),
                ErrorCode::PatchInvalid,
            ),
            (
                json!([{"op": "add", "path": "/locks/-", "value": {}}]),
                ErrorCode::PatchInvalid,
            ),
            (
                json!([{"op": "replace", "path": "/pipeline/stage", "value": "verify"}]),
                ErrorCode::PatchInvalid,
            ),
            (
                json!([{"op": "replace", "path": "", "value": {}}]),
                ErrorCode::PatchInvalid,
            ),
            (
                json!([{"op": "move", "from": "/schemaVersion", "path": "/x"}]),
                ErrorCode::PatchInvalid,
            ),
            (
                json!([{"op": "remove", "path": "/tracks/9"}]),
                ErrorCode::PatchInvalid,
            ),
            (
                json!([{"op": "replace", "path": "/settings/fps", "value": "fast"}]),
                ErrorCode::PatchInvalid,
            ),
            (
                json!([{"op": "replace", "path": "/settings/fps", "value": 0}]),
                ErrorCode::Schema,
            ),
            (
                json!([{"op": "test", "path": "/name", "value": "Sample"}]),
                ErrorCode::PatchInvalid,
            ),
        ] {
            let err = apply(&p, &req(0, ops.clone())).unwrap_err();
            assert_eq!(err.code, code, "{ops}");
        }
    }

    #[test]
    fn whole_item_replace_cannot_forge_revs() {
        let p = sample();
        let mut item = serde_json::to_value(p.item("a").unwrap()).unwrap();
        item["rev"] = json!(99);
        item["name"] = json!("renamed");
        let out = apply(
            &p,
            &req(
                0,
                json!([{"op": "replace", "path": "/tracks/0/items/0", "value": item}]),
            ),
        )
        .unwrap();
        assert_eq!(out.project.item("a").unwrap().rev, 1);
    }
}

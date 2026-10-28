//! Schema migrations. Every schema change adds one step; older projects must keep loading (spec 3.3).

use serde_json::{Value, json};

use crate::error::{CkError, CkResult, ErrorCode};
use crate::model::{Project, SCHEMA_VERSION};

type Step = fn(&mut serde_json::Map<String, Value>);

/// `STEPS[n]` migrates a version-`n` project to version `n + 1`.
const STEPS: &[Step] = &[v0_to_v1];

/// v0: files written before versioning, without `schemaVersion` (and possibly without `rev`).
fn v0_to_v1(p: &mut serde_json::Map<String, Value>) {
    p.entry("rev").or_insert(json!(0));
}

/// Bring a stored project up to `SCHEMA_VERSION`, then parse and validate it.
pub fn migrate(mut value: Value) -> CkResult<Project> {
    let fail = |msg: String| CkError::new(ErrorCode::Migration, msg);
    let obj = value
        .as_object_mut()
        .ok_or_else(|| fail("project file is not a JSON object".into()))?;
    let from = match obj.get("schemaVersion") {
        None => 0,
        Some(v) => v
            .as_u64()
            .ok_or_else(|| fail("schemaVersion is not a number".into()))?,
    };
    if from > u64::from(SCHEMA_VERSION) {
        return Err(fail(format!(
            "project schemaVersion {from} is newer than this build ({SCHEMA_VERSION})"
        )));
    }
    for (n, step) in STEPS.iter().enumerate().skip(from as usize) {
        step(obj);
        obj.insert("schemaVersion".into(), json!(n + 1));
    }
    let project: Project = serde_json::from_value(value).map_err(|e| {
        fail(format!(
            "project does not match schema {SCHEMA_VERSION}: {e}"
        ))
    })?;
    project.validate()?;
    Ok(project)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_cover_every_version() {
        assert_eq!(STEPS.len(), SCHEMA_VERSION as usize);
    }

    #[test]
    fn current_projects_pass_through() {
        let p = crate::model::tests::sample();
        assert_eq!(migrate(serde_json::to_value(&p).unwrap()).unwrap(), p);
    }

    #[test]
    fn newer_and_malformed_files_are_refused() {
        let mut v = serde_json::to_value(crate::model::tests::sample()).unwrap();
        v["schemaVersion"] = json!(SCHEMA_VERSION + 1);
        assert_eq!(migrate(v).unwrap_err().code, ErrorCode::Migration);
        assert_eq!(
            migrate(json!([1, 2])).unwrap_err().code,
            ErrorCode::Migration
        );
        assert_eq!(
            migrate(json!({"schemaVersion": "one"})).unwrap_err().code,
            ErrorCode::Migration
        );
    }
}

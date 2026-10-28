//! Every stored project fixture, from any schema version, must load and validate.

use ck_core::migrate::migrate;
use ck_core::model::SCHEMA_VERSION;

#[test]
fn every_fixture_version_loads() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/projects");
    let mut seen = 0;
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let project = migrate(value).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        assert_eq!(project.schema_version, SCHEMA_VERSION, "{}", path.display());
        seen += 1;
    }
    assert!(seen >= 1, "no project fixtures found in {}", dir.display());
}

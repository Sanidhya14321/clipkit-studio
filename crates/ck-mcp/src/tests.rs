use std::collections::BTreeSet;

use ck_server::ops::caps;

use crate::server::tool_list;
use crate::tools::specs;

#[test]
fn tool_table_snapshot() {
    insta::assert_json_snapshot!("tool_table", specs());
}

#[test]
fn every_tool_is_named_after_its_cli_twin() {
    let mut names = BTreeSet::new();
    for s in specs() {
        let path = s.command.strip_prefix("ck ").unwrap();
        assert_eq!(s.name, path.replace(' ', "_"), "{}", s.command);
        assert!(names.insert(s.name), "duplicate tool {}", s.name);
        // MCP tool names: letters, digits, `_` and `-`, at most 64 chars.
        assert!(s.name.len() <= 64);
        assert!(
            s.name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        );
    }
}

#[test]
fn every_tool_has_an_object_schema_and_its_cli_cap() {
    let expected = [
        ("status", caps::STATUS),
        ("project_get", caps::SUMMARY.max(caps::LAYER)),
        ("project_patch", caps::PATCH),
        ("lock", caps::LOCK),
        ("unlock", caps::LOCK),
        ("heartbeat", caps::LOCK),
    ];
    let table: Vec<(&str, usize)> = specs().iter().map(|s| (s.name, s.cap)).collect();
    assert_eq!(table, expected);
    for s in specs() {
        assert_eq!(s.input_schema["type"], "object", "{}", s.name);
        assert!(!s.description.is_empty());
    }
}

#[test]
fn rmcp_tool_list_matches_the_table() {
    let listed: Vec<String> = tool_list().iter().map(|t| t.name.to_string()).collect();
    let table: Vec<String> = specs().iter().map(|s| s.name.to_owned()).collect();
    assert_eq!(listed, table);
}

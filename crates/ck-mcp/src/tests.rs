use std::collections::BTreeSet;

use ck_assets::library;
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
        ("studio", caps::STUDIO),
        ("status", caps::STATUS),
        ("project_get", caps::SUMMARY.max(caps::LAYER)),
        ("project_patch", caps::PATCH),
        ("lock", caps::LOCK),
        ("unlock", caps::LOCK),
        ("heartbeat", caps::LOCK),
        ("brief_propose", caps::BRIEF_PROPOSE),
        ("brief_wait", caps::BRIEF),
        ("brief_diff", caps::BRIEF),
        ("brief_get", caps::BRIEF),
        ("taste_search", library::caps::TASTE_SEARCH),
        ("taste_read", library::caps::TASTE_READ),
        ("look_list", library::caps::LOOK_LIST),
        ("look_get", library::caps::LOOK_GET),
        ("recipe_search", library::caps::RECIPE_SEARCH),
        ("recipe_card", library::caps::RECIPE_CARD),
        ("sources_list", ck_assets::sources::caps::SOURCES_LIST),
        ("sources_add", ck_assets::sources::caps::SOURCES_WRITE),
        ("sources_vet", ck_assets::vet::caps::VET),
        ("assets_search", ck_assets::retrieve::caps::SEARCH),
        ("assets_card", ck_assets::retrieve::caps::CARD),
        ("assets_skeleton", ck_assets::retrieve::caps::SKELETON),
        ("assets_slice", ck_assets::retrieve::caps::SLICE),
        ("assets_source", ck_assets::retrieve::caps::SOURCE),
        ("license_report", ck_assets::report::caps::REPORT),
        ("eval", ck_eval::caps::EVAL),
        ("replay", ck_server::debug::caps::REPLAY),
        ("bisect", ck_server::debug::caps::BISECT),
        ("trace_bundle", ck_server::debug::caps::TRACE),
        ("demo_reset", ck_server::demo::caps::DEMO),
        ("bench_report", ck_eval::bench::caps::REPORT),
        ("profile", ck_server::profile::caps::PROFILE),
        ("sources_ingest", ck_assets::ingest::caps::INGEST),
        ("sources_gaps", ck_assets::sources::caps::SOURCES_GAPS),
        ("look_apply", caps::LOOK_APPLY),
        ("recipe_apply", caps::RECIPE_APPLY),
        ("layout_check", caps::LAYOUT_CHECK),
        ("verify", caps::VERIFY),
        ("sound_plan", caps::SOUND_PLAN),
        ("doctor", ck_server::doctor::caps::DOCTOR),
        ("content_map", caps::CONTENT_MAP),
        ("grid_build", caps::GRID),
        ("plan_get", caps::PLAN),
        ("plan_set", caps::PLAN_WRITE),
        ("plan_candidates", caps::PLAN_CANDIDATES),
        ("plan_expand", caps::PLAN_WRITE),
        ("pipeline_detect", caps::PIPELINE),
        ("pipeline_next", caps::PIPELINE),
        ("handoff", caps::HANDOFF),
        ("import", caps::IMPORT),
        ("transcribe", caps::TRANSCRIBE),
        ("transcript_get", caps::TRANSCRIPT),
        ("cut_suggest", caps::CUT_SUGGEST),
        ("cut_apply", caps::CUT_APPLY),
        ("assets_derive", caps::DERIVE),
        ("render_check", caps::RENDER_CHECK),
        ("music_analyze", caps::MUSIC),
        ("reference_analyze", caps::REFERENCE),
        ("storyboard_get", caps::STORYBOARD),
        ("history", ck_server::session::branch::caps::HISTORY),
        ("branch_save", ck_server::session::branch::caps::BRANCH),
        ("branch_list", ck_server::session::branch::caps::BRANCH),
        ("branch_switch", ck_server::session::branch::caps::BRANCH),
        ("variants", ck_server::session::branch::caps::VARIANTS),
        ("export", caps::EXPORT),
        ("jobs", caps::EXPORT_JOBS),
        ("voice_script", caps::VOICE_SCRIPT),
        ("voice_mode", caps::VOICE_WRITE),
        ("voice_speak", caps::VOICE_WRITE),
        ("voice_clone", caps::VOICE_WRITE),
        ("voice_captions", caps::VOICE_SCRIPT),
        ("ask", caps::ASK),
        ("context", caps::CONTEXT),
        ("comments_list", caps::COMMENTS),
        ("comments_add", caps::COMMENT_WRITE),
        ("comments_reply", caps::COMMENT_WRITE),
        ("comments_resolve", caps::COMMENT_WRITE),
        ("wait", caps::WAIT),
    ];
    let table: Vec<(&str, usize)> = specs().iter().map(|s| (s.name, s.cap)).collect();
    assert_eq!(table, expected);
    for s in specs() {
        assert_eq!(s.input_schema["type"], "object", "{}", s.name);
        assert!(!s.description.is_empty());
    }
}

#[test]
fn offline_tools_are_exactly_the_library_tools() {
    for s in specs() {
        let offline = crate::tools::offline(
            std::path::Path::new("."),
            s.name,
            serde_json::json!({ "id": "nope" }),
        );
        assert_eq!(
            offline.is_some(),
            crate::tools::is_offline(s.name),
            "{}",
            s.name
        );
    }
    assert!(
        crate::tools::offline(std::path::Path::new("."), "status", serde_json::Value::Null)
            .is_none()
    );
}

#[test]
fn rmcp_tool_list_matches_the_table() {
    let listed: Vec<String> = tool_list().iter().map(|t| t.name.to_string()).collect();
    let table: Vec<String> = specs().iter().map(|s| s.name.to_owned()).collect();
    assert_eq!(listed, table);
}

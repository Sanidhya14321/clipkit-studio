use axum::http::{Request, StatusCode};
use ck_core::storage::Store;
use serde_json::json;

use super::fixture::{Fixture, HOST, TOKEN, body_of, rename};
use crate::auth::TOKEN_HEADER;

fn get(uri: &str) -> axum::http::request::Builder {
    Request::builder().method("GET").uri(uri)
}

#[tokio::test]
async fn status_with_token() {
    let f = Fixture::new();
    let (code, body) = f.call("GET", "/api/status", None).await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(body["projectId"], "p1");
    assert_eq!(body["rev"], 0);
    assert_eq!(body["agent"]["status"], "idle");
}

#[tokio::test]
async fn missing_or_wrong_token_is_refused() {
    let f = Fixture::new();
    let no_token = get("/api/status").header("host", HOST);
    let (code, body) = f.send(no_token.body(body_of(None)).unwrap()).await;
    assert_eq!(
        (code, body["code"].as_str()),
        (StatusCode::FORBIDDEN, Some("E_AUTH"))
    );

    let wrong = get("/api/status")
        .header("host", HOST)
        .header(TOKEN_HEADER, "test-tokeX");
    let (code, body) = f.send(wrong.body(body_of(None)).unwrap()).await;
    assert_eq!(
        (code, body["code"].as_str()),
        (StatusCode::FORBIDDEN, Some("E_AUTH"))
    );

    let ws_no_token = get("/ws").header("host", HOST);
    let (code, _) = f.send(ws_no_token.body(body_of(None)).unwrap()).await;
    assert_eq!(code, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn token_in_query_only_counts_for_ws() {
    let f = Fixture::new();
    let req = get(&format!("/api/status?token={TOKEN}")).header("host", HOST);
    let (code, body) = f.send(req.body(body_of(None)).unwrap()).await;
    assert_eq!(
        (code, body["code"].as_str()),
        (StatusCode::FORBIDDEN, Some("E_AUTH"))
    );
    // On /ws the query token passes the guard; the oneshot request is not a real upgrade, so
    // anything but 403 proves the guard let it through.
    let req = get(&format!("/ws?token={TOKEN}")).header("host", HOST);
    let (code, _) = f.send(req.body(body_of(None)).unwrap()).await;
    assert_ne!(code, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn every_error_code_maps_to_an_error_status() {
    for code in ck_core::ErrorCode::ALL {
        let status = crate::http_error::status_of(*code);
        assert!(
            status.is_client_error() || status.is_server_error(),
            "{code}"
        );
    }
}

#[tokio::test]
async fn foreign_host_is_refused_even_with_token() {
    // DNS rebinding: evil.example resolves to 127.0.0.1, so the browser sends Host: evil.example.
    let f = Fixture::new();
    for host in ["evil.example:4000", "127.0.0.1:4001", "localhost", ""] {
        let req = get("/api/status")
            .header("host", host)
            .header(TOKEN_HEADER, TOKEN);
        let (code, body) = f.send(req.body(body_of(None)).unwrap()).await;
        assert_eq!(code, StatusCode::FORBIDDEN, "host {host:?}");
        assert_eq!(body["code"], "E_AUTH");
    }
}

#[tokio::test]
async fn origin_must_be_this_server() {
    let f = Fixture::new();
    for (origin, ok) in [
        ("http://127.0.0.1:4000", true),
        ("http://localhost:4000", true),
        ("https://evil.example", false),
        ("http://127.0.0.1:5173", false),
        ("null", false),
    ] {
        let req = get("/api/status")
            .header("host", HOST)
            .header("origin", origin)
            .header(TOKEN_HEADER, TOKEN);
        let (code, _) = f.send(req.body(body_of(None)).unwrap()).await;
        assert_eq!(code == StatusCode::OK, ok, "origin {origin}");
    }
}

#[tokio::test]
async fn unknown_route_still_checks_host() {
    let f = Fixture::new();
    let req = get("/").header("host", "evil.example");
    let (code, _) = f.send(req.body(body_of(None)).unwrap()).await;
    assert_eq!(code, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn project_views() {
    let f = Fixture::new();
    let (code, summary) = f.call("GET", "/api/project", None).await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(summary["tracks"][0]["items"][0]["id"], "title");
    assert_eq!(summary["tracks"][0]["items"][0]["type"], "clip");
    assert_eq!(summary["tracks"][0]["items"][0]["dur"], 2.0);
    assert_eq!(summary["durationSec"], 2.0);

    let (code, layer) = f.call("GET", "/api/project?layer=title", None).await;
    assert_eq!(
        (code, layer["name"].as_str()),
        (StatusCode::OK, Some("Title"))
    );

    let (code, full) = f.call("GET", "/api/project?view=full", None).await;
    assert_eq!(
        (code, full["schemaVersion"].as_u64()),
        (StatusCode::OK, Some(1))
    );

    let (code, err) = f.call("GET", "/api/project?layer=nope", None).await;
    assert_eq!(
        (code, err["code"].as_str()),
        (StatusCode::NOT_FOUND, Some("E_NOT_FOUND"))
    );

    let (code, err) = f.call("GET", "/api/project?view=everything", None).await;
    assert_eq!(
        (code, err["code"].as_str()),
        (StatusCode::BAD_REQUEST, Some("E_REQUEST"))
    );
}

#[tokio::test]
async fn agent_patch_needs_its_lock_and_is_stored() {
    let f = Fixture::new();
    let (code, err) = f
        .call("POST", "/api/patch", Some(rename(0, "agent", "Hook")))
        .await;
    assert_eq!(
        (code, err["code"].as_str()),
        (StatusCode::LOCKED, Some("E_LOCK_NOT_HELD"))
    );

    let lock = json!({ "holder": "agent", "layerIds": ["title"] });
    let (code, locks) = f.call("POST", "/api/lock", Some(lock)).await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(locks["locks"][0]["layerId"], "title");

    let (code, out) = f
        .call("POST", "/api/patch", Some(rename(0, "agent", "Hook")))
        .await;
    assert_eq!(code, StatusCode::OK, "{out}");
    assert_eq!(out, json!({ "rev": 1, "touched": ["title"] }));

    let stored = Store::new(f.dir.path()).load().unwrap();
    assert_eq!(stored.project().rev, 1);
    assert_eq!(stored.project().item("title").unwrap().name, "Hook");
    assert_eq!(
        stored.project().locks.len(),
        1,
        "locks are saved with the project"
    );
}

#[tokio::test]
async fn human_edit_is_refused_while_agent_holds_the_layer() {
    let f = Fixture::new();
    let lock = json!({ "holder": "agent", "layerIds": ["title"] });
    f.call("POST", "/api/lock", Some(lock)).await;

    let (code, err) = f
        .call("POST", "/api/patch", Some(rename(0, "human", "Mine")))
        .await;
    assert_eq!(
        (code, err["code"].as_str()),
        (StatusCode::LOCKED, Some("E_LOCK_HELD"))
    );
    assert_eq!(err["layerIds"], json!(["title"]));
    assert_eq!(f.session.status().unwrap().rev, 0, "nothing was applied");

    // A human cannot steal the lock either.
    let steal = json!({ "holder": "human", "layerIds": ["title"] });
    let (code, err) = f.call("POST", "/api/lock", Some(steal)).await;
    assert_eq!(
        (code, err["code"].as_str()),
        (StatusCode::LOCKED, Some("E_LOCK_HELD"))
    );

    // After unlock the human edit goes through.
    let unlock = json!({ "holder": "agent" });
    let (code, out) = f.call("POST", "/api/unlock", Some(unlock)).await;
    assert_eq!(
        (code, out["locks"].as_array().map(Vec::len)),
        (StatusCode::OK, Some(0))
    );
    let (code, _) = f
        .call("POST", "/api/patch", Some(rename(0, "human", "Mine")))
        .await;
    assert_eq!(code, StatusCode::OK);
}

#[tokio::test]
async fn expired_agent_lock_frees_the_layer() {
    let f = Fixture::new();
    let lock = json!({ "holder": "agent", "layerIds": ["title"] });
    f.call("POST", "/api/lock", Some(lock)).await;

    f.clock.advance(20_000);
    let (_, hb) = f
        .call("POST", "/api/heartbeat", Some(json!({ "holder": "agent" })))
        .await;
    assert_eq!(hb["locks"][0]["heartbeatAtMs"], 20_000);
    f.clock.advance(20_000);
    let (code, _) = f
        .call("POST", "/api/patch", Some(rename(0, "human", "Mine")))
        .await;
    assert_eq!(code, StatusCode::LOCKED, "heartbeat kept the lock alive");

    f.clock.advance(ck_core::locks::DEFAULT_TTL_MS);
    let (code, _) = f
        .call("POST", "/api/patch", Some(rename(0, "human", "Mine")))
        .await;
    assert_eq!(
        code,
        StatusCode::OK,
        "lock expired after the heartbeat timeout"
    );
}

#[tokio::test]
async fn stale_patch_is_a_conflict() {
    let f = Fixture::new();
    f.call("POST", "/api/patch", Some(rename(0, "human", "A")))
        .await;
    let (code, err) = f
        .call("POST", "/api/patch", Some(rename(0, "human", "B")))
        .await;
    assert_eq!(
        (code, err["code"].as_str()),
        (StatusCode::CONFLICT, Some("E_PATCH_STALE"))
    );
}

#[tokio::test]
async fn protected_field_patch_is_invalid() {
    let f = Fixture::new();
    let body = json!({
        "baseRev": 0, "author": "human",
        "ops": [{ "op": "replace", "path": "/locks", "value": [] }]
    });
    let (code, err) = f.call("POST", "/api/patch", Some(body)).await;
    assert_eq!(
        (code, err["code"].as_str()),
        (StatusCode::UNPROCESSABLE_ENTITY, Some("E_PATCH_INVALID"))
    );
}

#[tokio::test]
async fn malformed_and_oversized_bodies_are_bad_requests() {
    let f = Fixture::new();
    let (code, err) = f
        .call("POST", "/api/patch", Some(json!({ "ops": 3 })))
        .await;
    assert_eq!(
        (code, err["code"].as_str()),
        (StatusCode::BAD_REQUEST, Some("E_REQUEST"))
    );

    let huge = "x".repeat(crate::api::BODY_LIMIT + 1);
    let mut body = rename(0, "human", "A");
    body["label"] = json!(huge);
    let (code, err) = f.call("POST", "/api/patch", Some(body)).await;
    assert_eq!(
        (code, err["code"].as_str()),
        (StatusCode::BAD_REQUEST, Some("E_REQUEST"))
    );
    assert_eq!(f.session.status().unwrap().rev, 0);
}

#[cfg(unix)]
#[tokio::test]
async fn failed_save_leaves_memory_at_the_stored_state() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    let history = f.dir.path().join("history");
    std::fs::set_permissions(&history, std::fs::Permissions::from_mode(0o500)).unwrap();
    let (code, err) = f
        .call("POST", "/api/patch", Some(rename(0, "human", "A")))
        .await;
    std::fs::set_permissions(&history, std::fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(
        (code, err["code"].as_str()),
        (StatusCode::INTERNAL_SERVER_ERROR, Some("E_STORAGE"))
    );
    assert_eq!(
        f.session.status().unwrap().rev,
        0,
        "memory rolled back to disk"
    );
    let (code, _) = f
        .call("POST", "/api/patch", Some(rename(0, "human", "A")))
        .await;
    assert_eq!(
        code,
        StatusCode::OK,
        "the session keeps working after a recovered failure"
    );
}

//! Shared ops through the blocking client against a real server on a free loopback port.

use ck_core::ErrorCode;
use ck_core::model::Holder;
use serde_json::json;

use super::fixture::{Fixture, rename};
use crate::client::{Client, RESPONSE_LIMIT};
use crate::info::ServerInfo;
use crate::ops::{Ops, capped, caps, trimmed};
use crate::serve::Server;
use crate::session::{LockRequest, PatchResult, UnlockRequest};

/// A running server on its own runtime; dropping it stops the server.
struct Live {
    f: Fixture,
    info: ServerInfo,
    /// Kept alive for the server task; dropped after the stop signal.
    _rt: tokio::runtime::Runtime,
    stop: Option<tokio::sync::oneshot::Sender<()>>,
}

impl Live {
    fn start() -> Self {
        let f = Fixture::new();
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();
        let server = rt
            .block_on(Server::bind(f.dir.path(), f.session.clone(), 0))
            .unwrap();
        let info = server.info().clone();
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        rt.spawn(server.run(async {
            let _ = rx.await;
        }));
        Self {
            f,
            info,
            _rt: rt,
            stop: Some(tx),
        }
    }

    fn ops(&self) -> Ops {
        Ops::connect(self.f.dir.path()).unwrap()
    }
}

impl Drop for Live {
    fn drop(&mut self) {
        if let Some(tx) = self.stop.take() {
            let _ = tx.send(());
        }
    }
}

fn lock(holder: Holder, ids: &[&str]) -> LockRequest {
    LockRequest {
        holder,
        layer_ids: ids.iter().map(|s| (*s).to_owned()).collect(),
    }
}

#[test]
fn reads_status_summary_and_layer() {
    let live = Live::start();
    let ops = live.ops();
    let status = ops.status().unwrap();
    assert_eq!(status.project_id, "p1");
    assert_eq!(status.rev, 0);
    let summary = ops.summary().unwrap();
    assert_eq!(summary.tracks[0].items[0].id, "title");
    assert_eq!(ops.layer("title").unwrap().name, "Title");

    let missing = ops.layer("no such/id?&x=1").unwrap_err();
    assert_eq!(missing.code, ErrorCode::NotFound);
    assert_eq!(missing.layer_ids, ["no such/id?&x=1"], "query is encoded");
}

#[test]
fn patch_lock_unlock_round_trip() {
    let live = Live::start();
    let ops = live.ops();

    let locked = ops.lock(&lock(Holder::Agent, &["title"])).unwrap();
    assert_eq!(locked.locks.len(), 1);
    assert_eq!(ops.status().unwrap().locks.len(), 1);

    // The human is refused while the agent holds the layer; the error survives the HTTP trip.
    let human = serde_json::from_value(rename(0, "human", "Mine")).unwrap();
    let refused = ops.patch(&human).unwrap_err();
    assert_eq!(refused.code, ErrorCode::LockHeld);
    assert_eq!(refused.layer_ids, ["title"]);

    let agent = serde_json::from_value(rename(0, "agent", "Agent")).unwrap();
    let applied = ops.patch(&agent).unwrap();
    assert_eq!(
        (applied.rev, applied.touched.as_slice()),
        (1, &["title".to_owned()][..])
    );

    let stale = ops.patch(&agent).unwrap_err();
    assert_eq!(stale.code, ErrorCode::PatchStale);

    let released = ops
        .unlock(&UnlockRequest {
            holder: Holder::Agent,
            layer_ids: None,
        })
        .unwrap();
    assert!(released.locks.is_empty());
    assert!(ops.status().unwrap().locks.is_empty());
}

#[test]
fn missing_or_dead_server_is_e_server() {
    let dir = tempfile::tempdir().unwrap();
    let err = Ops::connect(dir.path()).err().unwrap();
    assert_eq!(err.code, ErrorCode::Server);

    // A stale server.json (nothing listening) also maps to E_SERVER, not a hang or panic.
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let ops = Ops::new(Client::new(&ServerInfo::new(port, "t".into())));
    assert_eq!(ops.status().unwrap_err().code, ErrorCode::Server);
}

#[test]
fn wrong_token_is_e_auth() {
    let live = Live::start();
    let mut info = live.info.clone();
    info.token = "wrong".into();
    let err = Ops::new(Client::new(&info)).status().unwrap_err();
    assert_eq!(err.code, ErrorCode::Auth);
}

#[test]
fn write_replies_over_the_cap_are_trimmed_not_refused() {
    let touched: Vec<String> = (0..1000).map(|i| format!("layer-{i:04}")).collect();
    let big = PatchResult {
        rev: 7,
        touched: touched.clone(),
        omitted: 0,
    };
    let out = trimmed("project patch", caps::PATCH, big).unwrap();
    assert!(serde_json::to_vec(&out).unwrap().len() <= caps::PATCH);
    assert_eq!(out.rev, 7);
    assert!(!out.touched.is_empty());
    assert_eq!(
        out.touched[..],
        touched[..out.touched.len()],
        "keeps a prefix"
    );
    assert_eq!(
        out.touched.len() + out.omitted,
        1000,
        "omitted counts the rest"
    );

    let small = PatchResult {
        rev: 1,
        touched: vec!["a".into()],
        omitted: 0,
    };
    let json = serde_json::to_value(trimmed("project patch", caps::PATCH, small).unwrap()).unwrap();
    assert!(
        json.get("omitted").is_none(),
        "omitted only appears when used"
    );

    // A large lock call through the server succeeds and reports how many locks were left out.
    let live = Live::start();
    let ids: Vec<String> = (0..500).map(|i| format!("layer-{i:04}")).collect();
    let ids: Vec<&str> = ids.iter().map(String::as_str).collect();
    let locked = live.ops().lock(&lock(Holder::Agent, &ids)).unwrap();
    assert_eq!(locked.locks.len() + locked.omitted, 500);
    assert!(locked.omitted > 0);
}

/// A one-shot raw HTTP server that answers any request with `reply`.
fn raw_server(reply: Vec<u8>) -> ServerInfo {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        let (mut sock, _) = listener.accept().unwrap();
        let mut buf = [0u8; 4096];
        let _ = sock.read(&mut buf);
        let _ = sock.write_all(&reply);
    });
    ServerInfo::new(port, "t".into())
}

#[test]
fn non_json_errors_and_oversize_replies_are_mapped() {
    let info = raw_server(b"HTTP/1.1 502 Bad Gateway\r\ncontent-length: 4\r\n\r\noops".to_vec());
    let err = Ops::new(Client::new(&info)).status().unwrap_err();
    assert_eq!(err.code, ErrorCode::Server);
    assert!(err.message.contains("502"), "{}", err.message);

    let body = format!("\"{}\"", "x".repeat(RESPONSE_LIMIT as usize));
    let mut reply =
        format!("HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n", body.len()).into_bytes();
    reply.extend_from_slice(body.as_bytes());
    let err = Ops::new(Client::new(&raw_server(reply)))
        .status()
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::OutputCap);
}

#[test]
fn answers_over_the_cap_are_refused() {
    let big = json!({ "text": "x".repeat(caps::LAYER) });
    let err = capped("project get --layer", caps::LAYER, big).unwrap_err();
    assert_eq!(err.code, ErrorCode::OutputCap);
    assert!(capped("status", caps::STATUS, json!({ "ok": true })).is_ok());

    // A real layer over its cap is refused end to end.
    let live = Live::start();
    let ops = live.ops();
    let long = json!({
        "baseRev": 0, "author": "human", "label": "long name",
        "ops": [{ "op": "replace", "path": "/tracks/0/items/0/name", "value": "n".repeat(caps::LAYER) }]
    });
    ops.patch(&serde_json::from_value(long).unwrap()).unwrap();
    assert_eq!(ops.layer("title").unwrap_err().code, ErrorCode::OutputCap);
}

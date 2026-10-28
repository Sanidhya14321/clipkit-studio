use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use ck_core::Project;
use ck_core::model::Mode;
use ck_core::storage::Store;
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::auth::{Guard, TOKEN_HEADER};
use crate::clock::Clock;
use crate::session::Session;

pub const PORT: u16 = 4000;
pub const TOKEN: &str = "test-token";
pub const HOST: &str = "127.0.0.1:4000";

/// A clock the test moves by hand.
#[derive(Default)]
pub struct FakeClock(AtomicU64);

impl FakeClock {
    pub fn advance(&self, ms: u64) {
        self.0.fetch_add(ms, Ordering::SeqCst);
    }
}

impl Clock for FakeClock {
    fn now_ms(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }
}

/// A stored project with one overlay track holding a clip `title` from 0 to 2 s.
pub fn project() -> Project {
    let mut p = Project::new("p1", "Demo", Mode::Generate);
    p.tracks = serde_json::from_value(json!([{
        "id": "t1",
        "kind": "overlay",
        "items": [{ "id": "title", "name": "Title", "type": "clip", "at": 0.0, "dur": 2.0 }]
    }]))
    .unwrap();
    p
}

pub struct Fixture {
    pub dir: tempfile::TempDir,
    pub clock: Arc<FakeClock>,
    pub session: Arc<Session>,
}

impl Fixture {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        Store::new(dir.path()).init(&project()).unwrap();
        let clock = Arc::new(FakeClock::default());
        let session = Arc::new(Session::open(dir.path(), clock.clone()).unwrap());
        Self {
            dir,
            clock,
            session,
        }
    }

    /// Call the router with the right Host and token.
    pub async fn call(&self, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
        let req = Request::builder()
            .method(method)
            .uri(uri)
            .header("host", HOST)
            .header(TOKEN_HEADER, TOKEN)
            .header("content-type", "application/json");
        self.send(req.body(body_of(body)).unwrap()).await
    }

    pub async fn send(&self, req: Request<Body>) -> (StatusCode, Value) {
        let router = crate::api::router(self.session.clone(), Guard::new(PORT, TOKEN));
        let resp = router.oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        (status, value)
    }
}

pub fn body_of(body: Option<Value>) -> Body {
    body.map_or_else(Body::empty, |v| Body::from(v.to_string()))
}

pub fn rename(base_rev: u64, author: &str, name: &str) -> Value {
    json!({
        "baseRev": base_rev,
        "author": author,
        "label": "rename title",
        "ops": [{ "op": "replace", "path": "/tracks/0/items/0/name", "value": name }]
    })
}

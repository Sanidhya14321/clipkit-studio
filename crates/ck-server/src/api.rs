//! HTTP routes. Handlers are thin: parse, run the session call off the async runtime, reply.

use std::sync::Arc;

use axum::extract::rejection::JsonRejection;
use axum::extract::{DefaultBodyLimit, Query, State};
use axum::http::{Method, Uri};
use axum::routing::{get, post};
use axum::{Json, Router, middleware};
use ck_core::patch::PatchRequest;
use ck_core::{CkError, CkResult, ErrorCode};
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::auth::{self, Guard};
use crate::http_error::ApiError;
use crate::session::{HeartbeatRequest, LockRequest, Session, UnlockRequest};
use crate::studio::{self, Assets};
use crate::ws;

/// Largest accepted request body. Patches are small; whole projects never travel inbound.
pub const BODY_LIMIT: usize = 1024 * 1024;

type ApiResult = Result<Json<serde_json::Value>, ApiError>;

/// The full router: `/api/*` and `/ws` need the token; everything passes the site guard; other
/// paths serve the embedded studio.
pub fn router(session: Arc<Session>, guard: Guard) -> Router {
    router_with_assets(session, guard, studio::embedded)
}

/// [`router`] with the studio files from `assets` (tests pass a fake build).
pub fn router_with_assets(session: Arc<Session>, guard: Guard, assets: Assets) -> Router {
    let api = Router::new()
        .route("/api/status", get(status))
        .route("/api/project", get(project))
        .route("/api/patch", post(patch))
        .route("/api/lock", post(lock))
        .route("/api/unlock", post(unlock))
        .route("/api/heartbeat", post(heartbeat))
        .route("/ws", get(ws::upgrade))
        .layer(middleware::from_fn_with_state(
            guard.clone(),
            auth::token_guard,
        ));
    Router::new()
        .merge(api)
        .fallback(move |method: Method, uri: Uri| studio::serve(assets, method, uri))
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
        .layer(middleware::from_fn_with_state(guard, auth::site_guard))
        .with_state(session)
}

/// `GET /api/project`: `?layer=<id>` for one layer, `?view=full` for the whole project (studio),
/// otherwise the compact summary.
#[derive(Debug, Default, Deserialize)]
struct ProjectQuery {
    layer: Option<String>,
    view: Option<String>,
}

async fn status(State(s): State<Arc<Session>>) -> ApiResult {
    blocking(s, |s| s.status()).await
}

async fn project(State(s): State<Arc<Session>>, Query(q): Query<ProjectQuery>) -> ApiResult {
    match (q.layer, q.view.as_deref()) {
        (Some(id), _) => blocking(s, move |s| s.layer(&id)).await,
        (None, Some("full")) => blocking(s, |s| s.project()).await,
        (None, None | Some("summary")) => blocking(s, |s| s.summary()).await,
        (None, Some(other)) => Err(ApiError(CkError::new(
            ErrorCode::Request,
            format!("unknown view {other:?}; use summary or full"),
        ))),
    }
}

async fn patch(
    State(s): State<Arc<Session>>,
    body: Result<Json<PatchRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.patch(&req)).await
}

async fn lock(
    State(s): State<Arc<Session>>,
    body: Result<Json<LockRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.lock(&req)).await
}

async fn unlock(
    State(s): State<Arc<Session>>,
    body: Result<Json<UnlockRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.unlock(&req)).await
}

async fn heartbeat(
    State(s): State<Arc<Session>>,
    body: Result<Json<HeartbeatRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.heartbeat(&req)).await
}

fn parse<T: DeserializeOwned>(body: Result<Json<T>, JsonRejection>) -> Result<T, ApiError> {
    body.map(|Json(v)| v)
        .map_err(|e| ApiError(CkError::new(ErrorCode::Request, e.body_text())))
}

/// Session calls fsync, so they run on the blocking pool, never on an async worker.
async fn blocking<T, F>(s: Arc<Session>, f: F) -> ApiResult
where
    T: serde::Serialize,
    F: FnOnce(&Session) -> CkResult<T> + Send + 'static,
    T: Send + 'static,
{
    let out = tokio::task::spawn_blocking(move || f(&s))
        .await
        .map_err(|e| ApiError(CkError::new(ErrorCode::Server, format!("task failed: {e}"))))??;
    serde_json::to_value(out)
        .map(Json)
        .map_err(|e| ApiError(CkError::new(ErrorCode::Server, e.to_string())))
}

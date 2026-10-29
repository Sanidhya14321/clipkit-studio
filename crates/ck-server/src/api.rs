//! HTTP routes. Handlers are thin: parse, run the session call off the async runtime, reply.

use std::sync::Arc;

use axum::extract::rejection::{JsonRejection, QueryRejection};
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
use crate::idle;
use crate::session::brief::{self, BriefDiffRequest, BriefProposeRequest, BriefWaitRequest};
use crate::session::handoff::{HandoffRequest, WaitRequest};
use crate::session::look::LookApplyRequest;
use crate::session::recipe::RecipeApplyRequest;
use crate::session::{HeartbeatRequest, LockRequest, Session, UnlockRequest};
use crate::studio::{self, Assets};
use crate::ws;
use tokio::sync::broadcast::error::RecvError;

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
        .route("/api/brief", get(brief_get))
        .route("/api/brief/propose", post(brief_propose))
        .route("/api/brief/diff", get(brief_diff))
        .route("/api/brief/wait", get(brief_wait))
        .route("/api/looks", get(looks))
        .route("/api/look/apply", post(look_apply))
        .route("/api/recipe/apply", post(recipe_apply))
        .route("/api/layout/check", get(layout_check))
        .route("/api/sound/plan", get(sound_plan))
        .route("/api/verify", get(verify))
        .route("/api/handoff", post(handoff))
        .route("/api/wait", get(wait))
        .route("/ws", get(ws::upgrade))
        // Runs after the token check, so only authorized calls keep the server alive.
        .layer(middleware::from_fn_with_state(session.clone(), idle::touch))
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

async fn brief_get(State(s): State<Arc<Session>>) -> ApiResult {
    blocking(s, |s| s.brief_get()).await
}

async fn brief_propose(
    State(s): State<Arc<Session>>,
    body: Result<Json<BriefProposeRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.brief_propose(&req)).await
}

async fn layout_check(State(s): State<Arc<Session>>) -> ApiResult {
    blocking(s, |s| s.layout_check()).await
}

async fn sound_plan(State(s): State<Arc<Session>>) -> ApiResult {
    blocking(s, |s| s.sound_plan()).await
}

async fn verify(State(s): State<Arc<Session>>) -> ApiResult {
    blocking(s, |s| s.verify()).await
}

async fn look_apply(
    State(s): State<Arc<Session>>,
    body: Result<Json<LookApplyRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.look_apply(&req)).await
}

async fn recipe_apply(
    State(s): State<Arc<Session>>,
    body: Result<Json<RecipeApplyRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.recipe_apply(&req)).await
}

async fn brief_diff(
    State(s): State<Arc<Session>>,
    q: Result<Query<BriefDiffRequest>, QueryRejection>,
) -> ApiResult {
    let req = query(q)?;
    blocking(s, move |s| s.brief_diff(&req)).await
}

/// Blocks until the brief is confirmed or auto, or the timeout passes. It subscribes before the
/// first check, so a confirmation landing in between still wakes it.
async fn brief_wait(
    State(s): State<Arc<Session>>,
    q: Result<Query<BriefWaitRequest>, QueryRejection>,
) -> ApiResult {
    let req = query(q)?;
    let secs = req
        .timeout_secs
        .unwrap_or(brief::WAIT_DEFAULT_SECS)
        .min(brief::WAIT_MAX_SECS);
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(secs);
    let mut rx = s.events();
    loop {
        let check = blocking(s.clone(), |s| s.brief_wait_check()).await?;
        if check.0["ready"] == true || tokio::time::Instant::now() >= deadline {
            return Ok(check);
        }
        match tokio::time::timeout_at(deadline, rx.recv()).await {
            // A change or a lagged receiver: check again. A timeout: the next check is the last.
            Ok(Ok(_)) | Ok(Err(RecvError::Lagged(_))) | Err(_) => {}
            Ok(Err(RecvError::Closed)) => return Ok(check),
        }
    }
}

async fn handoff(
    State(s): State<Arc<Session>>,
    body: Result<Json<HandoffRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.handoff(&req)).await
}

/// Blocks until the user makes a revision after `since`, or the timeout passes. `since` is
/// fixed before subscribing, and the check reads history, so nothing landing in between is lost.
async fn wait(
    State(s): State<Arc<Session>>,
    q: Result<Query<WaitRequest>, QueryRejection>,
) -> ApiResult {
    let req = query(q)?;
    let secs = req
        .timeout_secs
        .unwrap_or(brief::WAIT_DEFAULT_SECS)
        .min(brief::WAIT_MAX_SECS);
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(secs);
    let since = match req.since {
        Some(rev) => rev,
        None => blocking(s.clone(), |s| s.status()).await?.0["rev"]
            .as_u64()
            .unwrap_or(0),
    };
    let mut rx = s.events();
    loop {
        let check = blocking(s.clone(), move |s| s.wait_check(Some(since))).await?;
        let ready = check.0["events"].as_array().is_some_and(|e| !e.is_empty());
        if ready || tokio::time::Instant::now() >= deadline {
            return Ok(check);
        }
        match tokio::time::timeout_at(deadline, rx.recv()).await {
            Ok(Ok(_)) | Ok(Err(RecvError::Lagged(_))) | Err(_) => {}
            Ok(Err(RecvError::Closed)) => return Ok(check),
        }
    }
}

/// `GET /api/looks`: every built-in look with its tokens, for the Brief page's look cards
/// (four small files, well under any reply cap).
async fn looks() -> ApiResult {
    let looks = ck_assets::library::looks().map_err(ApiError)?;
    serde_json::to_value(looks)
        .map(Json)
        .map_err(|e| ApiError(CkError::new(ErrorCode::Server, e.to_string())))
}

fn query<T>(q: Result<Query<T>, QueryRejection>) -> Result<T, ApiError> {
    q.map(|Query(v)| v)
        .map_err(|e| ApiError(CkError::new(ErrorCode::Request, e.body_text())))
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

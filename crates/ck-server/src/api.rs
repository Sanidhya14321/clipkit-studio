//! HTTP routes. Handlers are thin: parse, run the session call off the async runtime, reply.

use axum::Extension;
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
use crate::session::ask::{AnswerRequest, AskRequest};
use crate::session::branch::{BranchRequest, HistoryRequest as RevHistoryRequest, VariantsRequest};
use crate::session::brief::{self, BriefDiffRequest, BriefProposeRequest, BriefWaitRequest};
use crate::session::comments::{
    CommentAddRequest, CommentListRequest, CommentReplyRequest, CommentResolveRequest,
};
use crate::session::context::StudioContext;
use crate::session::derive::DeriveRequest;
use crate::session::edit::{CutApplyRequest, CutWordsRequest, SourceRequest};
use crate::session::export::ExportRequest;
use crate::session::formats::TimelineExportRequest;
use crate::session::handoff::{HandoffRequest, WaitRequest};
use crate::session::history::{HistoryRequest, SplitRequest};
use crate::session::import::ImportRequest;
use crate::session::look::LookApplyRequest;
use crate::session::music::MusicAnalyzeRequest;
use crate::session::recipe::RecipeApplyRequest;
use crate::session::reference::ReferenceRequest;
use crate::session::render::RenderCheckRequest;
use crate::session::storyboard::StoryboardRequest;
use crate::session::verify::VerifyRequest;
use crate::session::voice::{
    ActiveTakeRequest, CaptionsRequest, ConsentRequest, MAX_RECORDING, SlotRequest, TakeQuery,
    VoiceCloneRequest, VoiceModeRequest, VoiceScriptRequest,
};
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
        .route("/api/import", post(import))
        .route("/api/transcribe", post(transcribe))
        .route("/api/transcript", get(transcript_get))
        .route("/api/cut/suggest", get(cut_suggest))
        .route("/api/cut/apply", post(cut_apply))
        .route("/api/export", post(export))
        .route("/api/export/jobs", get(export_jobs).post(export_enqueue))
        .route("/api/assets/derive", post(derive))
        .route("/api/render/check", post(render_check))
        .route("/api/history", get(history))
        .route("/api/export/timeline", post(export_timeline))
        .route("/api/storyboard/get", post(storyboard_get))
        // Studio only: the user's confirmation.
        .route("/api/storyboard/confirm", post(storyboard_confirm))
        .route(
            "/media/storyboard/latest.png",
            get(crate::media::storyboard_sheet),
        )
        .route("/api/reference/analyze", post(reference_analyze))
        .route("/api/music/analyze", post(music_analyze))
        .route("/api/branch/save", post(branch_save))
        .route("/api/branch/list", get(branch_list))
        .route("/api/branch/switch", post(branch_switch))
        .route("/api/variants", post(variants))
        .route("/api/comments", get(comments_list))
        .route("/api/voice/script", post(voice_script))
        .route("/api/voice/mode", post(voice_mode))
        .route("/api/voice/speak", post(voice_speak))
        // Studio only: record-along uploads and the takes list.
        .route(
            "/api/voice/take",
            post(voice_record).layer(DefaultBodyLimit::max(MAX_RECORDING)),
        )
        .route("/api/voice/active", post(voice_active_take))
        .route("/api/voice/consent", post(voice_consent))
        .route("/api/voice/clone", post(voice_clone))
        .route("/api/voice/captions", post(voice_captions))
        // Studio only: keyboard and push-to-talk commands.
        .route("/api/undo", post(undo))
        .route("/api/stop", post(stop))
        .route("/api/open-studio", post(open_studio))
        .route("/api/ask", post(ask))
        .route("/api/asks", get(asks_open))
        .route("/api/ask/answer", post(ask_answer))
        .route("/api/redo", post(redo))
        .route("/api/split", post(split))
        .route("/api/context", get(context).post(set_context))
        .route("/api/comments/add", post(comments_add))
        .route("/api/comments/reply", post(comments_reply))
        .route("/api/comments/resolve", post(comments_resolve))
        // Studio only: the full word list and delete-to-cut as a human edit.
        .route("/api/transcript/words", get(transcript_words))
        .route("/api/cut/words", post(cut_words))
        .route("/api/recipe/apply", post(recipe_apply))
        .route("/api/layout/check", get(layout_check))
        .route("/api/sound/plan", get(sound_plan))
        .route("/api/content/map", get(content_map))
        .route("/api/grid/build", post(grid_build))
        .route("/api/plan", get(plan_get).post(plan_set))
        .route("/api/plan/candidates", get(plan_candidates))
        .route("/api/plan/expand", post(plan_expand))
        .route("/api/verify", get(verify).post(verify_critic))
        .route("/api/handoff", post(handoff))
        .route("/api/pipeline/detect", post(pipeline_detect))
        .route("/api/pipeline/next", post(pipeline_next))
        .route("/api/wait", get(wait))
        .route("/ws", get(ws::upgrade))
        .route("/media/{name}", get(crate::media::proxy))
        .route("/media/source/{hash}", get(crate::media::original))
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

/// Human acts need the studio token: an agent token may not claim to be the user.
fn studio_only(role: crate::auth::Role) -> Result<(), ApiError> {
    if role == crate::auth::Role::Studio {
        return Ok(());
    }
    Err(ApiError(ck_core::CkError::new(
        ck_core::ErrorCode::Auth,
        "only the studio can act as the user",
    )))
}

async fn open_studio(State(s): State<Arc<Session>>) -> ApiResult {
    blocking(s, |s| s.open_studio().map(|()| serde_json::json!({}))).await
}

async fn patch(
    State(s): State<Arc<Session>>,
    Extension(role): Extension<crate::auth::Role>,
    body: Result<Json<PatchRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    if req.author != ck_core::model::Author::Agent {
        studio_only(role)?;
    }
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

async fn content_map(
    State(s): State<Arc<Session>>,
    Query(q): Query<crate::session::edit::SourceRequest>,
) -> ApiResult {
    blocking(s, move |s| s.content_map(&q)).await
}

async fn grid_build(
    State(s): State<Arc<Session>>,
    Json(r): Json<crate::session::edl::GridBuildRequest>,
) -> ApiResult {
    blocking(s, move |s| s.grid_build(&r)).await
}

async fn plan_get(State(s): State<Arc<Session>>) -> ApiResult {
    blocking(s, |s| s.plan_get()).await
}

async fn plan_set(
    State(s): State<Arc<Session>>,
    Json(r): Json<crate::session::edl::PlanSetRequest>,
) -> ApiResult {
    blocking(s, move |s| s.plan_set(&r)).await
}

async fn plan_candidates(State(s): State<Arc<Session>>) -> ApiResult {
    blocking(s, |s| s.plan_candidates()).await
}

async fn plan_expand(
    State(s): State<Arc<Session>>,
    Json(r): Json<crate::session::edl::PlanExpandRequest>,
) -> ApiResult {
    blocking(s, move |s| s.plan_expand(&r)).await
}

async fn sound_plan(State(s): State<Arc<Session>>) -> ApiResult {
    blocking(s, |s| s.sound_plan()).await
}

async fn verify(State(s): State<Arc<Session>>) -> ApiResult {
    blocking(s, |s| s.verify(&VerifyRequest::default())).await
}

/// `verify --critic`: the gates plus the critic's contact sheet (renders, so a POST).
async fn verify_critic(
    State(s): State<Arc<Session>>,
    body: Result<Json<VerifyRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.verify(&req)).await
}

async fn import(
    State(s): State<Arc<Session>>,
    body: Result<Json<ImportRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.import(&req)).await
}

async fn transcribe(
    State(s): State<Arc<Session>>,
    body: Result<Json<SourceRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.transcribe(&req)).await
}

async fn transcript_get(
    State(s): State<Arc<Session>>,
    q: Result<Query<SourceRequest>, QueryRejection>,
) -> ApiResult {
    let req = query(q)?;
    blocking(s, move |s| s.transcript_get(&req)).await
}

async fn cut_suggest(
    State(s): State<Arc<Session>>,
    q: Result<Query<SourceRequest>, QueryRejection>,
) -> ApiResult {
    let req = query(q)?;
    blocking(s, move |s| s.cut_suggest(&req)).await
}

async fn cut_apply(
    State(s): State<Arc<Session>>,
    body: Result<Json<CutApplyRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.cut_apply(&req)).await
}

async fn transcript_words(
    State(s): State<Arc<Session>>,
    q: Result<Query<SourceRequest>, QueryRejection>,
) -> ApiResult {
    let req = query(q)?;
    blocking(s, move |s| s.transcript_words(&req)).await
}

async fn cut_words(
    State(s): State<Arc<Session>>,
    body: Result<Json<CutWordsRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.cut_words(&req)).await
}

async fn export_jobs(State(s): State<Arc<Session>>) -> ApiResult {
    blocking(s, |s| s.export_jobs()).await
}

async fn export_enqueue(
    State(s): State<Arc<Session>>,
    body: Result<Json<ExportRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s.clone(), move |_| s.export_enqueue(&req)).await
}

async fn export(
    State(s): State<Arc<Session>>,
    body: Result<Json<ExportRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.export(&req)).await
}

async fn comments_list(
    State(s): State<Arc<Session>>,
    q: Result<Query<CommentListRequest>, QueryRejection>,
) -> ApiResult {
    let req = query(q)?;
    blocking(s, move |s| s.comments_list(&req)).await
}

async fn comments_add(
    State(s): State<Arc<Session>>,
    body: Result<Json<CommentAddRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.comments_add(&req)).await
}

async fn comments_reply(
    State(s): State<Arc<Session>>,
    body: Result<Json<CommentReplyRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.comments_reply(&req)).await
}

async fn comments_resolve(
    State(s): State<Arc<Session>>,
    body: Result<Json<CommentResolveRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.comments_resolve(&req)).await
}

async fn context(State(s): State<Arc<Session>>) -> ApiResult {
    blocking(s, |s| s.context()).await
}

/// Studio only: the pointer state `ck context` reports.
async fn set_context(
    State(s): State<Arc<Session>>,
    body: Result<Json<StudioContext>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.set_context(req)).await
}

async fn voice_script(
    State(s): State<Arc<Session>>,
    body: Result<Json<VoiceScriptRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.voice_script(&req)).await
}

async fn voice_mode(
    State(s): State<Arc<Session>>,
    body: Result<Json<VoiceModeRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.voice_mode(&req)).await
}

async fn voice_speak(
    State(s): State<Arc<Session>>,
    body: Result<Json<SlotRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.voice_speak(&req)).await
}

async fn voice_record(
    State(s): State<Arc<Session>>,
    q: Result<Query<TakeQuery>, QueryRejection>,
    headers: axum::http::HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult {
    let req = query(q)?;
    let kind = headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let ext = if kind.starts_with("audio/ogg") {
        "ogg"
    } else if kind.starts_with("audio/wav") || kind.starts_with("audio/x-wav") {
        "wav"
    } else {
        "webm"
    };
    blocking(s, move |s| s.voice_record(&req.slot, &body, ext)).await
}

async fn voice_active_take(
    State(s): State<Arc<Session>>,
    body: Result<Json<ActiveTakeRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.voice_active_take(&req)).await
}

async fn voice_consent(
    State(s): State<Arc<Session>>,
    Extension(role): Extension<crate::auth::Role>,
    body: Result<Json<ConsentRequest>, JsonRejection>,
) -> ApiResult {
    studio_only(role)?;
    let req = parse(body)?;
    blocking(s, move |s| s.voice_consent(&req)).await
}

async fn voice_clone(
    State(s): State<Arc<Session>>,
    body: Result<Json<VoiceCloneRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.voice_clone(&req)).await
}

async fn voice_captions(
    State(s): State<Arc<Session>>,
    body: Result<Json<CaptionsRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.voice_captions(&req)).await
}

async fn undo(
    State(s): State<Arc<Session>>,
    body: Result<Json<HistoryRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.undo(&req)).await
}

async fn redo(
    State(s): State<Arc<Session>>,
    body: Result<Json<HistoryRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.redo(&req)).await
}

async fn split(
    State(s): State<Arc<Session>>,
    body: Result<Json<SplitRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.split(&req)).await
}

/// Blocks until the studio answers or the timeout passes (like `wait`); subscribes before the
/// first check so an answer landing in between still wakes it.
async fn ask(
    State(s): State<Arc<Session>>,
    body: Result<Json<AskRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    let secs = req
        .timeout_secs
        .unwrap_or(brief::WAIT_DEFAULT_SECS)
        .min(brief::WAIT_MAX_SECS);
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(secs);
    let mut rx = s.events();
    let open = req.clone();
    let id = blocking(s.clone(), move |s| s.ask_open(&open)).await?.0;
    let id = id.as_str().unwrap_or_default().to_owned();
    loop {
        let check_id = id.clone();
        let check = blocking(s.clone(), move |s| s.ask_check(&check_id)).await?;
        if check.0["answered"] == true || tokio::time::Instant::now() >= deadline {
            return Ok(check);
        }
        match tokio::time::timeout_at(deadline, rx.recv()).await {
            Ok(Ok(_)) | Ok(Err(RecvError::Lagged(_))) | Err(_) => {}
            Ok(Err(RecvError::Closed)) => return Ok(check),
        }
    }
}

async fn asks_open(State(s): State<Arc<Session>>) -> ApiResult {
    blocking(s, |s| s.asks_open()).await
}

async fn ask_answer(
    State(s): State<Arc<Session>>,
    body: Result<Json<AnswerRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.ask_answer(&req)).await
}

async fn derive(
    State(s): State<Arc<Session>>,
    body: Result<Json<DeriveRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.derive(&req)).await
}

async fn render_check(
    State(s): State<Arc<Session>>,
    body: Result<Json<RenderCheckRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.render_check(&req)).await
}

async fn history(
    State(s): State<Arc<Session>>,
    q: Result<Query<RevHistoryRequest>, QueryRejection>,
) -> ApiResult {
    let req = query(q)?;
    blocking(s, move |s| s.history(&req)).await
}

async fn branch_save(
    State(s): State<Arc<Session>>,
    body: Result<Json<BranchRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.branch_save(&req)).await
}

async fn branch_list(State(s): State<Arc<Session>>) -> ApiResult {
    blocking(s, |s| s.branch_list()).await
}

async fn branch_switch(
    State(s): State<Arc<Session>>,
    body: Result<Json<BranchRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.branch_switch(&req)).await
}

async fn variants(
    State(s): State<Arc<Session>>,
    body: Result<Json<VariantsRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.variants(&req)).await
}

async fn music_analyze(
    State(s): State<Arc<Session>>,
    body: Result<Json<MusicAnalyzeRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.music_analyze(&req)).await
}

async fn reference_analyze(
    State(s): State<Arc<Session>>,
    body: Result<Json<ReferenceRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.reference_analyze(&req)).await
}

async fn storyboard_get(
    State(s): State<Arc<Session>>,
    body: Result<Json<StoryboardRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.storyboard_get(&req)).await
}

async fn storyboard_confirm(State(s): State<Arc<Session>>) -> ApiResult {
    blocking(s, |s| s.storyboard_confirm()).await
}

async fn export_timeline(
    State(s): State<Arc<Session>>,
    body: Result<Json<TimelineExportRequest>, JsonRejection>,
) -> ApiResult {
    let req = parse(body)?;
    blocking(s, move |s| s.export_timeline(&req)).await
}

/// Studio only: the Stop button.
async fn stop(
    State(s): State<Arc<Session>>,
    Extension(role): Extension<crate::auth::Role>,
) -> ApiResult {
    studio_only(role)?;
    blocking(s, |s| s.request_stop()).await
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

async fn pipeline_detect(State(s): State<Arc<Session>>) -> ApiResult {
    blocking(s, move |s| s.pipeline_detect()).await
}

async fn pipeline_next(State(s): State<Arc<Session>>) -> ApiResult {
    blocking(s, move |s| s.pipeline_next()).await
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

//! Shared agent operations (spec 6.2). The CLI and the MCP server are thin adapters over these
//! functions, so both get the same inputs, outputs and size caps.

use std::path::Path;

use ck_assets::layout::LayoutIssue;
use ck_core::model::{Item, Lock};
use ck_core::patch::PatchRequest;
use ck_core::{CkError, CkResult, ErrorCode};
use serde::Serialize;

use crate::client::Client;
use crate::session::ask::{AskRequest, AskResult};
use crate::session::brief::{
    BriefChange, BriefDiff, BriefDiffRequest, BriefProposeRequest, BriefProposeResult, BriefView,
    BriefWaitRequest, BriefWaitResult,
};
use crate::session::comments::{
    CommentAddRequest, CommentBundle, CommentListRequest, CommentReplyRequest,
    CommentResolveRequest, CommentResult, CommentView,
};
use crate::session::context::{ContextView, OpenComment};
use crate::session::edit::{
    CutApplyRequest, CutApplyResult, CutSuggestResult, SourceRequest, TranscribeResult,
    TranscriptView,
};
use crate::session::export::{ExportRequest, ExportResult};
use crate::session::handoff::{HandoffRequest, HandoffResult, WaitEvent, WaitRequest, WaitResult};
use crate::session::import::{ImportRequest, ImportResult};
use crate::session::layout::LayoutCheckResult;
use crate::session::look::{LookApplyRequest, LookApplyResult};
use crate::session::recipe::{RecipeApplyRequest, RecipeApplyResult};
use crate::session::sound::{SoundCue, SoundPlan};
use crate::session::verify::{VerifyIssue, VerifyResult};
use crate::session::voice::{
    CaptionsRequest, CaptionsResult, SlotCaption, SlotRequest, SlotSummary, TakeResult,
    VoiceCloneRequest, VoiceCloneResult, VoiceModeRequest, VoiceModeResult, VoiceScriptRequest,
    VoiceScriptResult,
};
use crate::session::{HeartbeatRequest, LockRequest, LockResult, PatchResult, UnlockRequest};
use crate::view::{Status, Summary};

/// Documented output size caps in bytes of compact JSON (spec 9.1). A read over its cap is
/// refused with E_OUTPUT_CAP instead of flooding the agent's context; a write reply is trimmed.
pub mod caps {
    pub const STATUS: usize = 4 * 1024;
    pub const SUMMARY: usize = 16 * 1024;
    pub const LAYER: usize = 8 * 1024;
    pub const PATCH: usize = 4 * 1024;
    pub const LOCK: usize = 4 * 1024;
    pub const STUDIO: usize = 1024;
    /// The confirmed brief reaches the agent in about 600 tokens (spec 8.1).
    pub const BRIEF: usize = 4 * 1024;
    pub const BRIEF_PROPOSE: usize = 1024;
    pub const RECIPE_APPLY: usize = 1024;
    pub const LOOK_APPLY: usize = 2 * 1024;
    pub const LAYOUT_CHECK: usize = 4 * 1024;
    pub const VERIFY: usize = 4 * 1024;
    pub const SOUND_PLAN: usize = 4 * 1024;
    pub const HANDOFF: usize = 1024;
    pub const WAIT: usize = 4 * 1024;
    pub const IMPORT: usize = 1024;
    pub const TRANSCRIBE: usize = 1024;
    /// About 12 KB per 10 minutes of speech (spec 8); longer sources are trimmed.
    pub const TRANSCRIPT: usize = 16 * 1024;
    pub const CUT_SUGGEST: usize = 8 * 1024;
    pub const CUT_APPLY: usize = 2 * 1024;
    pub const EXPORT: usize = 1024;
    /// The send bundle: open comments and the layers they point at.
    pub const COMMENTS: usize = 8 * 1024;
    pub const COMMENT_WRITE: usize = 2 * 1024;
    pub const VOICE_SCRIPT: usize = 4 * 1024;
    pub const VOICE_WRITE: usize = 1024;
    pub const ASK: usize = 1024;
    /// About 300 tokens (spec 6.4).
    pub const CONTEXT: usize = 1536;
}

pub struct Ops {
    client: Client,
}

impl Ops {
    /// Connect to the studio serving the project under `root` (the `.clipkit` folder).
    pub fn connect(root: &Path) -> CkResult<Self> {
        Ok(Self::new(Client::connect(root)?))
    }

    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// `status`: rev, mode, pipeline stage, agent state and live locks.
    pub fn status(&self) -> CkResult<Status> {
        capped("status", caps::STATUS, self.client.get("/api/status", &[])?)
    }

    /// `project get --summary`: ids, kinds and spans per track.
    pub fn summary(&self) -> CkResult<Summary> {
        let summary = self.client.get("/api/project", &[("view", "summary")])?;
        capped("project get --summary", caps::SUMMARY, summary)
    }

    /// `project get --layer <id>`: one item in full.
    pub fn layer(&self, id: &str) -> CkResult<Item> {
        let item = self.client.get("/api/project", &[("layer", id)])?;
        capped("project get --layer", caps::LAYER, item)
    }

    /// `project patch`: apply an RFC 6902 patch at `baseRev`.
    pub fn patch(&self, req: &PatchRequest) -> CkResult<PatchResult> {
        trimmed(
            "project patch",
            caps::PATCH,
            self.client.post("/api/patch", req)?,
        )
    }

    /// `lock`: acquire layer locks; returns the holder's live locks.
    pub fn lock(&self, req: &LockRequest) -> CkResult<LockResult> {
        trimmed("lock", caps::LOCK, self.client.post("/api/lock", req)?)
    }

    /// `unlock`: release some or all of the holder's locks.
    pub fn unlock(&self, req: &UnlockRequest) -> CkResult<LockResult> {
        trimmed("unlock", caps::LOCK, self.client.post("/api/unlock", req)?)
    }

    /// `heartbeat`: renew every live lock of the holder; returns them.
    pub fn heartbeat(&self, req: &HeartbeatRequest) -> CkResult<LockResult> {
        let reply = self.client.post("/api/heartbeat", req)?;
        trimmed("heartbeat", caps::LOCK, reply)
    }

    /// `brief propose`: write the inferred values; the studio shows the Brief page.
    pub fn brief_propose(&self, req: &BriefProposeRequest) -> CkResult<BriefProposeResult> {
        req.check()?;
        let reply = self.client.post("/api/brief/propose", req)?;
        trimmed("brief propose", caps::BRIEF_PROPOSE, reply)
    }

    /// `brief get`: the brief in compact form with its rev.
    pub fn brief_get(&self) -> CkResult<BriefView> {
        capped(
            "brief get",
            caps::BRIEF,
            self.client.get("/api/brief", &[])?,
        )
    }

    /// `brief wait`: block until the user confirms the brief (or it is auto), up to the timeout.
    pub fn brief_wait(&self, req: &BriefWaitRequest) -> CkResult<BriefWaitResult> {
        let secs = req.timeout_secs.map(|s| s.to_string());
        let query: Vec<(&str, &str)> = secs.iter().map(|s| ("timeoutSecs", s.as_str())).collect();
        capped(
            "brief wait",
            caps::BRIEF,
            self.client.get("/api/brief/wait", &query)?,
        )
    }

    /// `brief diff --since <rev>`: only what changed in the brief.
    pub fn brief_diff(&self, req: &BriefDiffRequest) -> CkResult<BriefDiff> {
        let since = req.since.to_string();
        let reply = self.client.get("/api/brief/diff", &[("since", &since)])?;
        trimmed("brief diff", caps::BRIEF, reply)
    }

    /// `look apply`: re-theme the project by switching its look (one write of `tokens.look`);
    /// the reply lists the layers that change. `dryRun` only previews.
    pub fn look_apply(&self, req: &LookApplyRequest) -> CkResult<LookApplyResult> {
        let reply = self.client.post("/api/look/apply", req)?;
        trimmed("look apply", caps::LOOK_APPLY, reply)
    }

    /// `recipe apply`: expand a recipe with slot values into named layers, as one revision.
    pub fn recipe_apply(&self, req: &RecipeApplyRequest) -> CkResult<RecipeApplyResult> {
        let reply = self.client.post("/api/recipe/apply", req)?;
        trimmed("recipe apply", caps::RECIPE_APPLY, reply)
    }

    /// `layout check`: the layout rules the text layers break (safe area, overlap, gap, grid,
    /// size, cut text, too many blocks). Issues past the cap are counted in `omitted`.
    pub fn layout_check(&self) -> CkResult<LayoutCheckResult> {
        let reply = self.client.get("/api/layout/check", &[])?;
        trimmed("layout check", caps::LAYOUT_CHECK, reply)
    }

    /// `verify`: the quality gates of spec 4.3 that apply to a generated video. Issues past the
    /// cap are counted in `omitted`.
    pub fn verify(&self) -> CkResult<VerifyResult> {
        let reply = self.client.get("/api/verify", &[])?;
        trimmed("verify", caps::VERIFY, reply)
    }

    /// `sound plan`: whooshes before cuts, hits on text landings, risers before hooks, room
    /// tone under cuts and duck cues for music under voice. Cues past the cap are counted in
    /// `omitted`.
    pub fn sound_plan(&self) -> CkResult<SoundPlan> {
        let reply = self.client.get("/api/sound/plan", &[])?;
        trimmed("sound plan", caps::SOUND_PLAN, reply)
    }

    /// `import`: probe, hash, proxy and extract audio, then register the file as a source.
    pub fn import(&self, req: &ImportRequest) -> CkResult<ImportResult> {
        capped(
            "import",
            caps::IMPORT,
            self.client.post_slow("/api/import", req)?,
        )
    }

    /// `transcribe`: word-level transcript of a source, stored on disk.
    pub fn transcribe(&self, req: &SourceRequest) -> CkResult<TranscribeResult> {
        let reply = self.client.post_slow("/api/transcribe", req)?;
        capped("transcribe", caps::TRANSCRIBE, reply)
    }

    /// `transcript get`: the packed transcript. Lines past the cap are counted in `omitted`.
    pub fn transcript_get(&self, req: &SourceRequest) -> CkResult<TranscriptView> {
        let reply = self
            .client
            .get("/api/transcript", &[("source", req.source.as_str())])?;
        trimmed("transcript get", caps::TRANSCRIPT, reply)
    }

    /// `cut suggest`: cut candidates on word boundaries and scene changes.
    pub fn cut_suggest(&self, req: &SourceRequest) -> CkResult<CutSuggestResult> {
        let reply = self
            .client
            .get("/api/cut/suggest", &[("source", req.source.as_str())])?;
        trimmed("cut suggest", caps::CUT_SUGGEST, reply)
    }

    /// `cut apply`: the chosen cuts become footage segments on the video track.
    pub fn cut_apply(&self, req: &CutApplyRequest) -> CkResult<CutApplyResult> {
        let reply = self.client.post("/api/cut/apply", req)?;
        trimmed("cut apply", caps::CUT_APPLY, reply)
    }

    /// `export`: the fast ffmpeg path to an MP4 with a platform preset and a loudness gate.
    pub fn export(&self, req: &ExportRequest) -> CkResult<ExportResult> {
        capped(
            "export",
            caps::EXPORT,
            self.client.post_slow("/api/export", req)?,
        )
    }

    /// `comments list`: the send bundle (open and working comments, their layers in short).
    /// Comments past the cap are counted in `omitted`.
    pub fn comments_list(&self, req: &CommentListRequest) -> CkResult<CommentBundle> {
        let all = if req.all { "true" } else { "false" };
        let reply = self.client.get("/api/comments", &[("all", all)])?;
        trimmed("comments list", caps::COMMENTS, reply)
    }

    /// `comments add`: a comment anchored to a moment, range, layer or element.
    pub fn comments_add(&self, req: &CommentAddRequest) -> CkResult<CommentResult> {
        let reply = self.client.post("/api/comments/add", req)?;
        trimmed("comments add", caps::COMMENT_WRITE, reply)
    }

    /// `comments reply`: an agent reply starts work on the comment, optionally with a scope.
    pub fn comments_reply(&self, req: &CommentReplyRequest) -> CkResult<CommentResult> {
        let reply = self.client.post("/api/comments/reply", req)?;
        trimmed("comments reply", caps::COMMENT_WRITE, reply)
    }

    /// `comments resolve`: closes the comment; lists the layers changed since it was made.
    pub fn comments_resolve(&self, req: &CommentResolveRequest) -> CkResult<CommentResult> {
        let reply = self.client.post("/api/comments/resolve", req)?;
        trimmed("comments resolve", caps::COMMENT_WRITE, reply)
    }

    /// `context`: the studio selection, hovered element, playhead, loop range and open
    /// comments. Comments past the cap are counted in `omitted`.
    pub fn context(&self) -> CkResult<ContextView> {
        trimmed(
            "context",
            caps::CONTEXT,
            self.client.get("/api/context", &[])?,
        )
    }

    /// `voice script`: the script as voice slots with target windows (replaces the slots).
    pub fn voice_script(&self, req: &VoiceScriptRequest) -> CkResult<VoiceScriptResult> {
        let reply = self.client.post("/api/voice/script", req)?;
        trimmed("voice script", caps::VOICE_SCRIPT, reply)
    }

    /// `voice speak`: AI or cloned speech for a slot as its new active take (or the
    /// placeholder when no speech can be made).
    pub fn voice_speak(&self, req: &SlotRequest) -> CkResult<TakeResult> {
        let reply = self.client.post_slow("/api/voice/speak", req)?;
        capped("voice speak", caps::VOICE_WRITE, reply)
    }

    /// `voice clone`: a cloned voice from a consented studio recording.
    pub fn voice_clone(&self, req: &VoiceCloneRequest) -> CkResult<VoiceCloneResult> {
        let reply = self.client.post_slow("/api/voice/clone", req)?;
        capped("voice clone", caps::VOICE_WRITE, reply)
    }

    /// `voice captions`: captions from each slot's active take timing.
    pub fn voice_captions(&self, req: &CaptionsRequest) -> CkResult<CaptionsResult> {
        let reply = self.client.post_slow("/api/voice/captions", req)?;
        trimmed("voice captions", caps::VOICE_SCRIPT, reply)
    }

    /// `voice mode`: switch a slot between ai, recorded and cloned.
    pub fn voice_mode(&self, req: &VoiceModeRequest) -> CkResult<VoiceModeResult> {
        let reply = self.client.post("/api/voice/mode", req)?;
        capped("voice mode", caps::VOICE_WRITE, reply)
    }

    /// `ask`: show a question with options in the studio and block (at most 25 s) for the
    /// answer; unanswered, the reply carries the id to keep waiting with.
    pub fn ask(&self, req: &AskRequest) -> CkResult<AskResult> {
        capped("ask", caps::ASK, self.client.post("/api/ask", req)?)
    }

    /// `handoff`: release every agent lock and set the agent idle; the user takes over.
    pub fn handoff(&self, req: &HandoffRequest) -> CkResult<HandoffResult> {
        capped(
            "handoff",
            caps::HANDOFF,
            self.client.post("/api/handoff", req)?,
        )
    }

    /// `wait`: block until the user changes something after `since` (comments, brief edits,
    /// confirmations, layer edits), up to the timeout. Events past the cap are counted in
    /// `omitted`; the agent reads them with the next `wait`.
    pub fn wait(&self, req: &WaitRequest) -> CkResult<WaitResult> {
        let since = req.since.map(|s| s.to_string());
        let secs = req.timeout_secs.map(|s| s.to_string());
        let mut query: Vec<(&str, &str)> = Vec::new();
        if let Some(s) = &since {
            query.push(("since", s));
        }
        if let Some(s) = &secs {
            query.push(("timeoutSecs", s));
        }
        let mut result: WaitResult =
            trimmed("wait", caps::WAIT, self.client.get("/api/wait", &query)?)?;
        if result.omitted > 0 {
            result.rev = resume_rev(&result.events, req.since.unwrap_or(0));
        }
        Ok(result)
    }
}

/// The rev to resume from after trimming: the last kept event, so the next `wait` with it as
/// `since` returns the events that were left out.
fn resume_rev(kept: &[WaitEvent], since: u64) -> u64 {
    kept.last().map_or(since, |e| e.rev)
}

impl Trim for WaitResult {
    type Entry = WaitEvent;
    fn parts(&mut self) -> (&mut Vec<WaitEvent>, &mut usize) {
        (&mut self.events, &mut self.omitted)
    }
}

impl Trim for TranscriptView {
    type Entry = String;
    fn parts(&mut self) -> (&mut Vec<String>, &mut usize) {
        (&mut self.packed.lines, &mut self.packed.omitted)
    }
}

impl Trim for CutSuggestResult {
    type Entry = ck_voice::CutCandidate;
    fn parts(&mut self) -> (&mut Vec<ck_voice::CutCandidate>, &mut usize) {
        (&mut self.cuts, &mut self.omitted)
    }
}

impl Trim for CutApplyResult {
    type Entry = String;
    fn parts(&mut self) -> (&mut Vec<String>, &mut usize) {
        (&mut self.segments, &mut self.omitted)
    }
}

impl Trim for CommentBundle {
    type Entry = CommentView;
    fn parts(&mut self) -> (&mut Vec<CommentView>, &mut usize) {
        (&mut self.comments, &mut self.omitted)
    }
}

impl Trim for CommentResult {
    type Entry = String;
    fn parts(&mut self) -> (&mut Vec<String>, &mut usize) {
        (&mut self.changed, &mut self.omitted)
    }
}

impl Trim for ContextView {
    type Entry = OpenComment;
    fn parts(&mut self) -> (&mut Vec<OpenComment>, &mut usize) {
        (&mut self.open_comments, &mut self.omitted)
    }
}

impl Trim for VoiceScriptResult {
    type Entry = SlotSummary;
    fn parts(&mut self) -> (&mut Vec<SlotSummary>, &mut usize) {
        (&mut self.slots, &mut self.omitted)
    }
}

impl Trim for CaptionsResult {
    type Entry = SlotCaption;
    fn parts(&mut self) -> (&mut Vec<SlotCaption>, &mut usize) {
        (&mut self.captions, &mut self.omitted)
    }
}

impl Trim for SoundPlan {
    type Entry = SoundCue;
    fn parts(&mut self) -> (&mut Vec<SoundCue>, &mut usize) {
        (&mut self.cues, &mut self.omitted)
    }
}

impl Trim for VerifyResult {
    type Entry = VerifyIssue;
    fn parts(&mut self) -> (&mut Vec<VerifyIssue>, &mut usize) {
        (&mut self.issues, &mut self.omitted)
    }
}

impl Trim for LayoutCheckResult {
    type Entry = LayoutIssue;
    fn parts(&mut self) -> (&mut Vec<LayoutIssue>, &mut usize) {
        (&mut self.issues, &mut self.omitted)
    }
}

impl Trim for LookApplyResult {
    type Entry = String;
    fn parts(&mut self) -> (&mut Vec<String>, &mut usize) {
        (&mut self.impacted, &mut self.omitted)
    }
}

impl Trim for RecipeApplyResult {
    type Entry = String;
    fn parts(&mut self) -> (&mut Vec<String>, &mut usize) {
        (&mut self.layers, &mut self.omitted)
    }
}

impl Trim for BriefProposeResult {
    type Entry = String;
    fn parts(&mut self) -> (&mut Vec<String>, &mut usize) {
        (&mut self.kept, &mut self.omitted)
    }
}

impl Trim for BriefDiff {
    type Entry = BriefChange;
    fn parts(&mut self) -> (&mut Vec<BriefChange>, &mut usize) {
        (&mut self.changes, &mut self.omitted)
    }
}

/// A write reply whose list can be shortened, counting what was left out.
pub trait Trim {
    type Entry: Serialize;
    fn parts(&mut self) -> (&mut Vec<Self::Entry>, &mut usize);
}

impl Trim for PatchResult {
    type Entry = String;
    fn parts(&mut self) -> (&mut Vec<String>, &mut usize) {
        (&mut self.touched, &mut self.omitted)
    }
}

impl Trim for LockResult {
    type Entry = Lock;
    fn parts(&mut self) -> (&mut Vec<Lock>, &mut usize) {
        (&mut self.locks, &mut self.omitted)
    }
}

/// Bytes kept free for the `omitted` field once a list is trimmed.
const OMITTED_ROOM: usize = 32;

/// Writes are committed before the reply exists, so an oversize write reply is never refused
/// (a retry would then fail as stale): its list keeps the prefix that fits and `omitted` counts
/// the rest.
pub fn trimmed<T: Serialize + Trim>(what: &str, cap: usize, mut value: T) -> CkResult<T> {
    if json_len(&value)? <= cap {
        return Ok(value);
    }
    let entries = std::mem::take(value.parts().0);
    let total = entries.len();
    let mut budget = cap.saturating_sub(json_len(&value)? + OMITTED_ROOM);
    let mut kept = Vec::new();
    for entry in entries {
        let len = json_len(&entry)? + 1; // plus the comma
        if len > budget {
            break;
        }
        budget -= len;
        kept.push(entry);
    }
    let (list, omitted) = value.parts();
    *omitted += total - kept.len();
    *list = kept;
    capped(what, cap, value)
}

/// Refuse `value` if its compact JSON exceeds `cap` bytes.
pub fn capped<T: Serialize>(what: &str, cap: usize, value: T) -> CkResult<T> {
    let len = json_len(&value)?;
    if len > cap {
        return Err(CkError::new(
            ErrorCode::OutputCap,
            format!("{what} answer is {len} bytes; the cap is {cap}"),
        ));
    }
    Ok(value)
}

fn json_len<T: Serialize>(value: &T) -> CkResult<usize> {
    serde_json::to_vec(value)
        .map(|v| v.len())
        .map_err(|e| CkError::new(ErrorCode::Server, e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::handoff::WaitKind;
    use ck_core::model::Author;

    fn event(rev: u64) -> WaitEvent {
        WaitEvent {
            rev,
            kind: WaitKind::Edit,
            author: Author::Human,
            label: String::new(),
            touched: Vec::new(),
        }
    }

    #[test]
    fn trimmed_wait_resumes_after_the_last_kept_event() {
        assert_eq!(resume_rev(&[event(4), event(7)], 3), 7);
        assert_eq!(resume_rev(&[], 3), 3);
    }
}

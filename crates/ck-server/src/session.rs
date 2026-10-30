//! One open project: the document, its store and the event bus. Every write goes through here,
//! so the lock gate and turn-taking hold no matter which client sends the change.
//!
//! Methods are synchronous (they fsync); async callers run them on `spawn_blocking`.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use ck_core::history::Revision;
use ck_core::model::{Author, Holder, Item, Lock};
use ck_core::patch::PatchRequest;
use ck_core::storage::Store;
use ck_core::{CkError, CkResult, Document, ErrorCode, Project};
use ck_voice::{FishStt, FishTts, Speaker, Transcriber};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use ts_rs::TS;

use crate::clock::Clock;
use crate::event::Event;
use crate::idle::Activity;
use crate::view::{Status, Summary};

pub mod ask;
pub mod branch;
pub mod brief;
pub mod comments;
pub mod context;
pub mod derive;
pub mod edit;
pub mod export;
pub mod formats;
pub mod handoff;
pub mod history;
pub mod import;
pub mod layout;
pub mod look;
pub mod recipe;
pub mod reference;
pub mod render;
pub mod sound;
pub mod storyboard;
pub mod verify;
pub mod voice;

/// Events buffered per subscriber before it lags (a lagging studio gets a fresh snapshot).
const BUS_CAPACITY: usize = 256;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct PatchResult {
    pub rev: u64,
    pub touched: Vec<String>,
    /// Touched ids left out of `touched` to keep the reply under its size cap.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub omitted: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct LockRequest {
    pub holder: Holder,
    pub layer_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct UnlockRequest {
    pub holder: Holder,
    /// Layers to release; omitted means every lock of the holder.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layer_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatRequest {
    pub holder: Holder,
}

/// The holder's live locks after a lock call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct LockResult {
    pub locks: Vec<Lock>,
    /// Locks left out of `locks` to keep the reply under its size cap.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub omitted: usize,
}

fn is_zero(n: &usize) -> bool {
    *n == 0
}

pub struct Session {
    store: Store,
    doc: Mutex<Document>,
    events: broadcast::Sender<Event>,
    clock: Arc<dyn Clock>,
    /// Set when a failed save could not be undone by reloading; every call then fails.
    degraded: AtomicBool,
    activity: Arc<Activity>,
    transcriber: Arc<dyn Transcriber>,
    speaker: Arc<dyn Speaker>,
    /// What the studio last reported pointing at (UI state, never stored).
    studio_context: Mutex<context::StudioContext>,
    /// The agent's open questions and their undelivered answers (`ck ask`).
    asks: Mutex<ask::Asks>,
    /// Taste profile file; None when there is no home folder.
    profile: Option<std::path::PathBuf>,
}

impl Session {
    /// Load the project stored under `root` (normally `<workdir>/.clipkit`).
    pub fn open(root: &Path, clock: Arc<dyn Clock>) -> CkResult<Self> {
        // Absolute, so paths handed to the render worker (which runs in its own folder) and
        // to ffmpeg never depend on the current directory.
        let root = std::path::absolute(root)
            .map_err(|e| CkError::new(ErrorCode::Storage, format!("{}: {e}", root.display())))?;
        let store = Store::new(&root);
        let doc = store.load()?;
        let (events, _) = broadcast::channel(BUS_CAPACITY);
        Ok(Self {
            store,
            doc: Mutex::new(doc),
            events,
            clock,
            degraded: AtomicBool::new(false),
            activity: Arc::default(),
            transcriber: Arc::new(FishStt::from_env()),
            speaker: Arc::new(FishTts::from_env()),
            studio_context: Mutex::default(),
            asks: Mutex::default(),
            // Unit tests never touch a real profile; they opt in with `with_profile`.
            profile: if cfg!(test) {
                None
            } else {
                crate::profile::default_path()
            },
        })
    }

    /// Use another speaker (tests, or a local voice once one is installed).
    pub fn with_speaker(mut self, speaker: Arc<dyn Speaker>) -> Self {
        self.speaker = speaker;
        self
    }

    /// Use another taste profile file (tests).
    pub fn with_profile(mut self, path: std::path::PathBuf) -> Self {
        self.profile = Some(path);
        self
    }

    /// No taste profile: neither learned from nor used (ck's own scripted edits).
    pub fn without_profile(mut self) -> Self {
        self.profile = None;
        self
    }

    pub fn profile_path(&self) -> Option<&Path> {
        self.profile.as_deref()
    }

    /// Learns from a human revision's brief changes (spec 8.8). Never fails the edit.
    fn learn_profile(&self, before: &ck_core::model::Brief, after: &ck_core::model::Brief) {
        let Some(path) = &self.profile else { return };
        let mut p = crate::profile::load(path);
        if p.enabled
            && p.learn(before, after)
            && let Err(e) = crate::profile::save(path, &p)
        {
            tracing::warn!(error = %e, "taste profile not saved");
        }
    }

    /// Use another transcriber (tests, or a local model once one is installed).
    pub fn with_transcriber(mut self, transcriber: Arc<dyn Transcriber>) -> Self {
        self.transcriber = transcriber;
        self
    }

    /// Subscribe to the bus and return the current snapshot. Subscribing first means no change
    /// lands between the snapshot and the first event (a duplicate snapshot is harmless).
    pub fn subscribe(&self) -> CkResult<(Event, broadcast::Receiver<Event>)> {
        let rx = self.events.subscribe();
        Ok((snapshot(&*self.doc()?), rx))
    }

    /// Subscribe to the bus without a snapshot (for calls that block until something changes).
    pub fn events(&self) -> broadcast::Receiver<Event> {
        self.events.subscribe()
    }

    /// Activity for idle shutdown (spec 5.1).
    pub fn activity(&self) -> &Arc<Activity> {
        &self.activity
    }

    /// The original file of the imported source with this content hash.
    pub fn source_file(&self, hash: &str) -> CkResult<std::path::PathBuf> {
        let doc = self.doc()?;
        let src = doc
            .project()
            .sources
            .iter()
            .find(|s| s.hash == hash)
            .ok_or_else(|| CkError::new(ErrorCode::NotFound, format!("no source {hash:?}")))?;
        let p = Path::new(&src.path);
        Ok(if p.is_absolute() {
            p.to_path_buf()
        } else {
            let root = self.store.root();
            root.parent().unwrap_or(root).join(p)
        })
    }

    /// `.clipkit/cache/`: proxies, extracted audio and transcripts.
    pub fn cache_dir(&self) -> std::path::PathBuf {
        self.store.root().join("cache")
    }

    pub fn now_ms(&self) -> u64 {
        self.clock.now_ms()
    }

    pub fn project(&self) -> CkResult<Project> {
        Ok(self.doc()?.project().clone())
    }

    pub fn status(&self) -> CkResult<Status> {
        Ok(Status::of(self.doc()?.project(), self.now_ms()))
    }

    pub fn summary(&self) -> CkResult<Summary> {
        Ok(Summary::of(self.doc()?.project()))
    }

    pub fn layer(&self, id: &str) -> CkResult<Item> {
        self.doc()?.project().item(id).cloned().ok_or_else(|| {
            CkError::new(ErrorCode::NotFound, format!("no layer with id {id:?}"))
                .with_layers([id.to_owned()])
        })
    }

    pub fn patch(&self, req: &PatchRequest) -> CkResult<PatchResult> {
        let now = self.now_ms();
        let mut doc = self.doc()?;
        let before = doc.project().brief.clone();
        let rev = doc.patch(req, now)?.clone();
        if req.author != Author::Agent {
            self.learn_profile(&before, &doc.project().brief);
        }
        self.persist(&mut doc)?;
        self.publish(Event::Revision {
            rev: rev.rev,
            author: rev.author,
            label: rev.label,
            touched: rev.touched.clone(),
        });
        self.publish_snapshot(&doc);
        Ok(PatchResult {
            rev: rev.rev,
            touched: rev.touched,
            omitted: 0,
        })
    }

    pub fn lock(&self, req: &LockRequest) -> CkResult<LockResult> {
        let now = self.now_ms();
        let mut doc = self.doc()?;
        doc.acquire(req.holder, &req.layer_ids, now)?;
        self.locks_changed(&mut doc, req.holder, now)
    }

    pub fn unlock(&self, req: &UnlockRequest) -> CkResult<LockResult> {
        let now = self.now_ms();
        let mut doc = self.doc()?;
        doc.release(req.holder, req.layer_ids.as_deref());
        self.locks_changed(&mut doc, req.holder, now)
    }

    pub fn heartbeat(&self, req: &HeartbeatRequest) -> CkResult<LockResult> {
        let now = self.now_ms();
        let mut doc = self.doc()?;
        doc.heartbeat(req.holder, now);
        self.locks_changed(&mut doc, req.holder, now)
    }

    fn locks_changed(
        &self,
        doc: &mut MutexGuard<'_, Document>,
        holder: Holder,
        now_ms: u64,
    ) -> CkResult<LockResult> {
        self.persist(doc)?;
        // Locks also move the agent between idle and working (spec 5.2): resend the project.
        self.publish_snapshot(doc);
        let locks = doc.project().locks.clone();
        self.publish(Event::Locks {
            locks: locks.clone(),
        });
        Ok(LockResult {
            locks: locks
                .into_iter()
                .filter(|l| l.holder == holder && l.is_live(now_ms))
                .collect(),
            omitted: 0,
        })
    }

    /// Commit `ops` as one agent revision, persist it and publish it. Like `brief propose`, the
    /// agent briefly takes the locks the write touches unless it holds them, so a human
    /// mid-edit refuses the write.
    fn commit_agent(
        &self,
        doc: &mut MutexGuard<'_, Document>,
        ops: serde_json::Value,
        label: String,
    ) -> CkResult<Revision> {
        self.commit_as(doc, Holder::Agent, ops, label, &[])
    }

    /// Commit as `holder` (agent or human): the locks the write touches, plus `extra` lock ids,
    /// are taken for the write unless the holder has them, so the other side's live locks
    /// refuse it.
    fn commit_as(
        &self,
        doc: &mut MutexGuard<'_, Document>,
        holder: Holder,
        ops: serde_json::Value,
        label: String,
        extra: &[String],
    ) -> CkResult<Revision> {
        let author = match holder {
            Holder::Agent => Author::Agent,
            Holder::Human => Author::Human,
        };
        self.commit_by(doc, author, ops, label, extra)
    }

    /// [`Session::commit_as`] with the author named (the voice lane writes as `voice` and holds
    /// locks as the human).
    fn commit_by(
        &self,
        doc: &mut MutexGuard<'_, Document>,
        author: Author,
        ops: serde_json::Value,
        label: String,
        extra: &[String],
    ) -> CkResult<Revision> {
        let holder = match author {
            Author::Agent => Holder::Agent,
            Author::Human | Author::Voice => Holder::Human,
        };
        let now = self.now_ms();
        let project = doc.project();
        let patch = PatchRequest {
            base_rev: project.rev,
            layer_revs: Default::default(),
            ops: serde_json::from_value(ops)
                .map_err(|e| CkError::new(ErrorCode::Server, e.to_string()))?,
            author,
            label,
        };
        // Dry run to learn the lock ids this write touches.
        let touched = ck_core::patch::apply(project, &patch)?.touched;
        let mut missing: Vec<String> = touched
            .into_iter()
            .chain(extra.iter().cloned())
            .filter(|id| {
                !project
                    .locks
                    .iter()
                    .any(|l| &l.layer_id == id && l.holder == holder && l.is_live(now))
            })
            .collect();
        missing.sort();
        missing.dedup();
        if !missing.is_empty() {
            doc.acquire(holder, &missing, now)?;
        }
        let committed = doc.patch(&patch, now).cloned();
        if !missing.is_empty() {
            doc.release(holder, Some(&missing));
        }
        let rev = committed?;
        self.persist(doc)?;
        self.publish(Event::Revision {
            rev: rev.rev,
            author: rev.author,
            label: rev.label.clone(),
            touched: rev.touched.clone(),
        });
        self.publish_snapshot(doc);
        Ok(rev)
    }

    /// Save, or on failure restore the in-memory document from disk so memory never runs ahead
    /// of what is stored.
    fn persist(&self, doc: &mut MutexGuard<'_, Document>) -> CkResult<()> {
        let Err(e) = self.store.save(doc) else {
            return Ok(());
        };
        tracing::error!(error = %e, "save failed; reloading from disk");
        match self.store.load() {
            Ok(stored) => **doc = stored,
            Err(reload) => {
                // Memory is ahead of disk and cannot be restored: refuse everything from now on.
                tracing::error!(error = %reload, "reload failed; session is degraded");
                self.degraded.store(true, Ordering::SeqCst);
            }
        }
        Err(e)
    }

    fn publish(&self, event: Event) {
        // No subscribers is normal (no studio open yet).
        let _ = self.events.send(event);
    }

    /// The snapshot clones the whole project, so only build it when someone listens.
    fn publish_snapshot(&self, doc: &Document) {
        if self.events.receiver_count() > 0 {
            self.publish(snapshot(doc));
        }
    }

    fn doc(&self) -> CkResult<MutexGuard<'_, Document>> {
        if self.degraded.load(Ordering::SeqCst) {
            return Err(CkError::new(
                ErrorCode::Storage,
                "a save failed and the project could not be reloaded from disk",
            )
            .with_hint("Restart ck studio; the stored project is the last good state."));
        }
        self.doc
            .lock()
            .map_err(|_| CkError::new(ErrorCode::Server, "project state lock poisoned"))
    }
}

fn snapshot(doc: &Document) -> Event {
    Event::Snapshot {
        project: Box::new(doc.project().clone()),
    }
}

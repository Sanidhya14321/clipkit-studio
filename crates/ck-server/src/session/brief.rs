//! Brief Studio essentials (spec 8.1): the agent proposes inferred values, the studio renders the
//! form, the user confirms, and the agent reads the confirmed brief and later only diffs.
//!
//! Every brief write is a normal revision through the patch, schema and lock gates, so undo,
//! history and turn-taking cover it like any other change.

use std::collections::BTreeMap;

use ck_core::model::{
    Author, Brief, BriefField, BriefQuestion, BriefSource, BriefStatus, Holder, MAX_BRIEF_QUESTIONS,
};
use ck_core::patch::{PROJECT_LAYER, PatchRequest};
use ck_core::{CkError, CkResult, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use super::{Session, is_zero};
use crate::event::Event;

/// Largest proposal in bytes of compact JSON: inferred values only, a few hundred tokens.
pub const PROPOSE_LIMIT: usize = 4 * 1024;
/// Most fields one proposal may carry.
pub const MAX_FIELDS: usize = 40;
const MAX_NAME: usize = 48;
/// Confidence given to a proposed value when the agent names none.
const DEFAULT_CONFIDENCE: f64 = 0.5;
/// `brief wait` blocks at most this long, below the client's 30 s timeout; the agent loops.
pub const WAIT_MAX_SECS: u64 = 25;
pub const WAIT_DEFAULT_SECS: u64 = 20;

/// `brief propose`: inferred values only. Fields the user already set are kept as they are.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BriefProposeRequest {
    /// Field name (goal, cta, audience, platform, aspect, length, pacing, look, voice, ...) to
    /// the inferred value.
    #[ts(as = "BTreeMap<String, Value>")]
    pub fields: BTreeMap<String, Value>,
    /// Optional confidence (0 to 1) per field; 0.5 when omitted.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub confidence: BTreeMap<String, f64>,
    /// At most three dynamic questions, each naming what it affects.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub questions: Vec<BriefQuestion>,
    /// The user asked for no questions or autopilot is on: mark the brief auto, so planning can
    /// start; the studio shows it afterwards as an editable card.
    #[serde(default)]
    pub auto: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct BriefProposeResult {
    pub rev: u64,
    pub status: BriefStatus,
    /// Proposed fields the user had already set; their values were kept.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kept: Vec<String>,
    /// Kept names left out of `kept` to keep the reply under its size cap.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub omitted: usize,
}

/// One field as the agent reads it: the value and where it came from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct BriefFieldView {
    #[ts(as = "Value")]
    pub value: Value,
    pub source: BriefSource,
}

/// `brief get`: the brief in compact form (no confidences) with the rev it was read at; pass
/// that rev to `brief diff` later.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct BriefView {
    pub rev: u64,
    pub status: BriefStatus,
    pub fields: BTreeMap<String, BriefFieldView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub questions: Vec<BriefQuestion>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub checkpoints: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license_mode: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BriefWaitRequest {
    /// Seconds to block (default 20, at most 25); 0 checks once.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
}

/// `brief wait`: `ready` once the user confirmed (or the brief is auto); then `brief` holds it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct BriefWaitResult {
    pub ready: bool,
    pub rev: u64,
    pub status: BriefStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brief: Option<BriefView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BriefDiffRequest {
    /// The rev of the brief the agent last read (from `brief get` or `brief wait`).
    pub since: u64,
}

/// One changed entry. `field` is a brief field name, `status`, `checkpoints`, `effort`,
/// `licenseMode`, or `question:<id>` for a question (its answer or presence).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct BriefChange {
    pub field: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(as = "Option<Value>")]
    pub from: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(as = "Option<Value>")]
    pub to: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<BriefSource>,
}

/// `brief diff`: what changed in the brief between `since` and `rev`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct BriefDiff {
    pub since: u64,
    pub rev: u64,
    pub changes: Vec<BriefChange>,
    /// Layers whose briefDeps name a changed field: the only ones to re-plan (spec 8.1).
    #[serde(default)]
    pub affected: Vec<String>,
    /// Changes left out of `changes` to keep the reply under its size cap.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub omitted: usize,
}

/// Layer ids, in track order, whose briefDeps name a changed field.
pub fn affected(project: &ck_core::Project, changes: &[BriefChange]) -> Vec<String> {
    project
        .tracks
        .iter()
        .flat_map(|t| &t.items)
        .filter(|i| {
            i.brief_deps
                .iter()
                .any(|d| changes.iter().any(|c| &c.field == d))
        })
        .map(|i| i.id.clone())
        .collect()
}

fn request_err(message: String) -> CkError {
    CkError::new(ErrorCode::Request, message)
}

impl BriefProposeRequest {
    /// Refuse proposals over the size, field and question limits before touching the project.
    pub fn check(&self) -> CkResult<()> {
        let len = serde_json::to_vec(self)
            .map_err(|e| CkError::new(ErrorCode::Server, e.to_string()))?
            .len();
        if len > PROPOSE_LIMIT {
            return Err(request_err(format!(
                "proposal is {len} bytes; the limit is {PROPOSE_LIMIT}: send inferred values only"
            )));
        }
        if self.fields.len() > MAX_FIELDS {
            return Err(request_err(format!(
                "proposal has {} fields; at most {MAX_FIELDS}",
                self.fields.len()
            )));
        }
        for name in self.fields.keys() {
            let ok = !name.is_empty()
                && name.len() <= MAX_NAME
                && name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'));
            if !ok {
                return Err(request_err(format!(
                    "field name {name:?} must be 1 to {MAX_NAME} letters, digits, _ - or ."
                )));
            }
        }
        for (name, c) in &self.confidence {
            if !self.fields.contains_key(name) {
                return Err(request_err(format!(
                    "confidence names {name:?}, which is not a proposed field"
                )));
            }
            if !(0.0..=1.0).contains(c) {
                return Err(request_err(format!(
                    "confidence of {name:?} must be 0 to 1"
                )));
            }
        }
        if self.questions.len() > MAX_BRIEF_QUESTIONS {
            return Err(request_err(format!(
                "{} questions; at most {MAX_BRIEF_QUESTIONS}",
                self.questions.len()
            )));
        }
        if self.questions.iter().any(|q| q.answer.is_some()) {
            return Err(request_err(
                "questions are answered by the user, not proposed with answers".into(),
            ));
        }
        Ok(())
    }

    /// The brief after this proposal: user-set fields win, other proposed fields become
    /// inferred, earlier inferred fields not proposed again are dropped, and answers to
    /// questions asked again unchanged are kept. Returns the brief and the kept field names.
    pub fn merge(&self, old: &Brief) -> (Brief, Vec<String>) {
        let mut next = old.clone();
        next.fields.retain(|_, f| f.source != BriefSource::Inferred);
        let mut kept = Vec::new();
        for (name, value) in &self.fields {
            if next
                .fields
                .get(name)
                .is_some_and(|f| f.source == BriefSource::User)
            {
                kept.push(name.clone());
                continue;
            }
            next.fields.insert(
                name.clone(),
                BriefField {
                    value: value.clone(),
                    source: BriefSource::Inferred,
                    confidence: self
                        .confidence
                        .get(name)
                        .copied()
                        .unwrap_or(DEFAULT_CONFIDENCE),
                },
            );
        }
        next.questions = self
            .questions
            .iter()
            .map(|q| {
                let answer = old
                    .questions
                    .iter()
                    .find(|o| o.id == q.id && o.text == q.text)
                    .and_then(|o| o.answer.clone());
                BriefQuestion {
                    answer,
                    ..q.clone()
                }
            })
            .collect();
        next.status = if self.auto {
            BriefStatus::Auto
        } else {
            BriefStatus::Proposed
        };
        (next, kept)
    }
}

impl BriefView {
    pub fn of(rev: u64, b: &Brief) -> Self {
        Self {
            rev,
            status: b.status,
            fields: b
                .fields
                .iter()
                .map(|(k, f)| {
                    (
                        k.clone(),
                        BriefFieldView {
                            value: f.value.clone(),
                            source: f.source,
                        },
                    )
                })
                .collect(),
            questions: b.questions.clone(),
            checkpoints: b.checkpoints.clone(),
            effort: b.effort.clone(),
            license_mode: b.license_mode.clone(),
        }
    }
}

/// Field-level changes from `old` to `new`, sorted by field name within each group.
pub fn changes(old: &Brief, new: &Brief) -> Vec<BriefChange> {
    let mut out = Vec::new();
    push(
        &mut out,
        "status".into(),
        Some(status_value(old.status)),
        Some(status_value(new.status)),
        None,
    );
    let names: std::collections::BTreeSet<&String> =
        old.fields.keys().chain(new.fields.keys()).collect();
    for name in names {
        let (a, b) = (old.fields.get(name), new.fields.get(name));
        // A source change alone (inferred to user with the same value) still matters to the
        // agent: the user now owns the field.
        let changed = a.map(|f| (&f.value, f.source)) != b.map(|f| (&f.value, f.source));
        if changed {
            out.push(BriefChange {
                field: name.clone(),
                from: a.map(|f| f.value.clone()),
                to: b.map(|f| f.value.clone()),
                source: b.map(|f| f.source),
            });
        }
    }
    let list = |v: &[String]| (!v.is_empty()).then(|| Value::from(v.to_vec()));
    let text = |v: &Option<String>| v.clone().map(Value::from);
    push(
        &mut out,
        "checkpoints".into(),
        list(&old.checkpoints),
        list(&new.checkpoints),
        None,
    );
    push(
        &mut out,
        "effort".into(),
        text(&old.effort),
        text(&new.effort),
        None,
    );
    push(
        &mut out,
        "licenseMode".into(),
        text(&old.license_mode),
        text(&new.license_mode),
        None,
    );
    let ids: std::collections::BTreeSet<&String> = old
        .questions
        .iter()
        .chain(&new.questions)
        .map(|q| &q.id)
        .collect();
    for id in ids {
        let find = |b: &Brief| {
            b.questions
                .iter()
                .find(|q| &q.id == id)
                .map(|q| serde_json::json!({ "text": q.text, "answer": q.answer }))
        };
        push(
            &mut out,
            format!("question:{id}"),
            find(old),
            find(new),
            None,
        );
    }
    out
}

fn push(
    out: &mut Vec<BriefChange>,
    field: String,
    from: Option<Value>,
    to: Option<Value>,
    source: Option<BriefSource>,
) {
    if from != to {
        out.push(BriefChange {
            field,
            from,
            to,
            source,
        });
    }
}

fn status_value(s: BriefStatus) -> Value {
    Value::from(match s {
        BriefStatus::Proposed => "proposed",
        BriefStatus::Confirmed => "confirmed",
        BriefStatus::Auto => "auto",
    })
}

impl Session {
    /// `brief propose`: write the inferred values as a revision by the agent. The agent briefly
    /// takes the project lock unless it holds it already, so a human mid-edit refuses the write
    /// (E_LOCK_HELD) instead of being overwritten.
    pub fn brief_propose(&self, req: &BriefProposeRequest) -> CkResult<BriefProposeResult> {
        req.check()?;
        let now = self.now_ms();
        let mut doc = self.doc()?;
        let (mut next, kept) = req.merge(&doc.project().brief);
        // Spec 8.8: the taste profile fills fields nobody set, never inferred or user values.
        if let Some(path) = self.profile_path() {
            let profile = crate::profile::load(path);
            if profile.enabled {
                for (field, (value, n)) in profile.defaults() {
                    next.fields.entry(field).or_insert(BriefField {
                        value,
                        source: BriefSource::Profile,
                        confidence: (0.5 + 0.1 * f64::from(n)).min(0.9),
                    });
                }
            }
        }
        if next == doc.project().brief {
            return Ok(BriefProposeResult {
                rev: doc.project().rev,
                status: next.status,
                kept,
                omitted: 0,
            });
        }
        let value = serde_json::to_value(&next)
            .map_err(|e| CkError::new(ErrorCode::Server, e.to_string()))?;
        let ops = serde_json::from_value(serde_json::json!([
            { "op": "replace", "path": "/brief", "value": value }
        ]))
        .map_err(|e| CkError::new(ErrorCode::Server, e.to_string()))?;
        let patch = PatchRequest {
            base_rev: doc.project().rev,
            layer_revs: Default::default(),
            ops,
            author: Author::Agent,
            label: "brief: propose".into(),
        };
        let project_lock = [PROJECT_LAYER.to_owned()];
        let held =
            doc.project().locks.iter().any(|l| {
                l.layer_id == PROJECT_LAYER && l.holder == Holder::Agent && l.is_live(now)
            });
        if !held {
            doc.acquire(Holder::Agent, &project_lock, now)?;
        }
        let committed = doc.patch(&patch, now).cloned();
        if !held {
            doc.release(Holder::Agent, Some(&project_lock));
        }
        let rev = committed?;
        self.persist(&mut doc)?;
        self.publish(Event::Revision {
            rev: rev.rev,
            author: rev.author,
            label: rev.label,
            touched: rev.touched,
        });
        self.publish_snapshot(&doc);
        Ok(BriefProposeResult {
            rev: rev.rev,
            status: next.status,
            kept,
            omitted: 0,
        })
    }

    /// `brief get`.
    pub fn brief_get(&self) -> CkResult<BriefView> {
        let doc = self.doc()?;
        Ok(BriefView::of(doc.project().rev, &doc.project().brief))
    }

    /// One `brief wait` check: ready once the brief is confirmed or auto.
    pub fn brief_wait_check(&self) -> CkResult<BriefWaitResult> {
        let view = self.brief_get()?;
        let ready = view.status != BriefStatus::Proposed;
        Ok(BriefWaitResult {
            ready,
            rev: view.rev,
            status: view.status,
            brief: ready.then_some(view),
        })
    }

    /// `brief diff --since <rev>`: the brief at `since` rebuilt from history, compared with now.
    pub fn brief_diff(&self, req: &BriefDiffRequest) -> CkResult<BriefDiff> {
        let doc = self.doc()?;
        let then = doc.content_at(req.since)?;
        let old: Brief = match then.get("brief") {
            Some(v) => serde_json::from_value(v.clone())
                .map_err(|e| CkError::new(ErrorCode::Server, format!("old brief: {e}")))?,
            None => Brief::default(),
        };
        let project = doc.project();
        let changes = changes(&old, &project.brief);
        Ok(BriefDiff {
            since: req.since,
            rev: project.rev,
            affected: affected(project, &changes),
            changes,
            omitted: 0,
        })
    }
}

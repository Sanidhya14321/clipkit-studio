//! Studio history and timeline commands used by push-to-talk (spec 6.4 Lane B) and keyboard:
//! undo and redo shared with the agent's history, and split at a time. Each runs as one
//! revision by the human or the voice lane; locks and the structure lock apply as for patches.

use ck_core::model::{Author, Item, ItemBody};
use ck_core::{CkError, CkResult, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use ts_rs::TS;

use super::{PatchResult, Session};
use crate::event::Event;

/// Shortest piece a split may leave, in seconds.
pub const MIN_PIECE: f64 = 0.05;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HistoryRequest {
    /// `human` (keyboard) or `voice` (push-to-talk); default human.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<Author>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SplitRequest {
    pub layer_id: String,
    /// Timeline time to split at, in seconds.
    pub at: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<Author>,
}

fn studio_author(a: Option<Author>) -> CkResult<Author> {
    match a.unwrap_or(Author::Human) {
        Author::Agent => Err(CkError::new(
            ErrorCode::Request,
            "studio commands are by the human or the voice lane",
        )),
        a => Ok(a),
    }
}

/// The two halves of `item` cut at timeline time `t`, or None when `t` is not inside it.
pub fn split_item(item: &Item, t: f64) -> Option<(Item, Item)> {
    let mut a = item.clone();
    let mut b = item.clone();
    b.id = format!("{}-b", item.id);
    match (&mut a.body, &mut b.body) {
        (
            ItemBody::Clip { at, dur, .. },
            ItemBody::Clip {
                at: bat, dur: bdur, ..
            },
        ) => {
            let cut = t - *at;
            if cut < MIN_PIECE || *dur - cut < MIN_PIECE {
                return None;
            }
            *bat = t;
            *bdur = *dur - cut;
            *dur = cut;
        }
        (
            ItemBody::Footage {
                at,
                src_out,
                speed,
                src_in,
                ..
            },
            ItemBody::Footage {
                at: bat,
                src_in: bin,
                ..
            },
        ) => {
            let len = (*src_out - *src_in) / *speed;
            let cut = t - *at;
            if cut < MIN_PIECE || len - cut < MIN_PIECE {
                return None;
            }
            let src_cut = *src_in + cut * *speed;
            *bat = t;
            *bin = src_cut;
            *src_out = src_cut;
        }
        _ => return None,
    }
    Some((a, b))
}

impl Session {
    fn history_step(&self, req: &HistoryRequest, redo: bool) -> CkResult<PatchResult> {
        let author = studio_author(req.author)?;
        let now = self.now_ms();
        let mut doc = self.doc()?;
        let rev = if redo {
            doc.redo(author, now)?.clone()
        } else {
            doc.undo(author, now)?.clone()
        };
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

    pub fn undo(&self, req: &HistoryRequest) -> CkResult<PatchResult> {
        self.history_step(req, false)
    }

    pub fn redo(&self, req: &HistoryRequest) -> CkResult<PatchResult> {
        self.history_step(req, true)
    }

    /// Split a clip or footage layer at a timeline time into `<id>` and `<id>-b`.
    pub fn split(&self, req: &SplitRequest) -> CkResult<PatchResult> {
        let author = studio_author(req.author)?;
        let mut doc = self.doc()?;
        let p = doc.project();
        let (ti, ii) = p
            .tracks
            .iter()
            .enumerate()
            .find_map(|(ti, t)| {
                t.items
                    .iter()
                    .position(|i| i.id == req.layer_id)
                    .map(|ii| (ti, ii))
            })
            .ok_or_else(|| {
                CkError::new(ErrorCode::NotFound, format!("no layer {:?}", req.layer_id))
            })?;
        let item = &p.tracks[ti].items[ii];
        if p.item(&format!("{}-b", item.id)).is_some() {
            return Err(CkError::new(
                ErrorCode::Request,
                format!("layer {:?} was already split here", item.id),
            ));
        }
        let (a, b) = split_item(item, req.at).ok_or_else(|| {
            CkError::new(
                ErrorCode::Request,
                format!("{:.2}s is not inside layer {:?}", req.at, item.id),
            )
        })?;
        let ops = json!([
            { "op": "replace", "path": format!("/tracks/{ti}/items/{ii}"), "value": a },
            { "op": "add", "path": format!("/tracks/{ti}/items/{}", ii + 1), "value": b }
        ]);
        let label = format!("split {} at {:.2}s", req.layer_id, req.at);
        let rev = self.commit_by(&mut doc, author, ops, label, &[])?;
        Ok(PatchResult {
            rev: rev.rev,
            touched: rev.touched,
            omitted: 0,
        })
    }
}

//! Events pushed to the studio over the WebSocket.

use ck_core::Project;
use ck_core::model::{Author, Lock};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// One message on the event bus. The studio renders from `snapshot` and narrates `revision`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Event {
    /// The whole project. Sent on connect and after every change.
    Snapshot { project: Box<Project> },
    /// A revision landed: drives the activity feed and the flash on touched layers.
    #[serde(rename_all = "camelCase")]
    Revision {
        rev: u64,
        author: Author,
        label: String,
        touched: Vec<String>,
    },
    /// Locks changed (acquire, release, heartbeat expiry).
    Locks { locks: Vec<Lock> },
}

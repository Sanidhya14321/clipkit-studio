//! Stable error codes. A code's string and meaning never change once shipped; add new codes instead.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Every error the engine can return, as a stable `E_*` code with a one-line hint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, TS)]
pub enum ErrorCode {
    /// The operation is not allowed in the current pipeline stage.
    #[serde(rename = "E_STAGE")]
    Stage,
    /// The same failing call was attempted too many times.
    #[serde(rename = "E_RETRY")]
    Retry,
    /// Another holder has the lock on a touched layer.
    #[serde(rename = "E_LOCK_HELD")]
    LockHeld,
    /// The agent patched a layer without holding its lock.
    #[serde(rename = "E_LOCK_NOT_HELD")]
    LockNotHeld,
    /// The patch was built against an older revision.
    #[serde(rename = "E_PATCH_STALE")]
    PatchStale,
    /// The patch could not be applied or touches a protected field.
    #[serde(rename = "E_PATCH_INVALID")]
    PatchInvalid,
    /// The project failed validation.
    #[serde(rename = "E_SCHEMA")]
    Schema,
    /// Undo or redo was requested with nothing to undo or redo.
    #[serde(rename = "E_HISTORY_EMPTY")]
    HistoryEmpty,
    /// Reading or writing project files failed.
    #[serde(rename = "E_STORAGE")]
    Storage,
    /// The project file is from an unsupported schema version.
    #[serde(rename = "E_MIGRATION")]
    Migration,
    /// The request lacked a valid session token or came from a foreign Host or Origin.
    #[serde(rename = "E_AUTH")]
    Auth,
    /// The requested layer, track or resource does not exist.
    #[serde(rename = "E_NOT_FOUND")]
    NotFound,
    /// The request body or arguments are malformed or too large.
    #[serde(rename = "E_REQUEST")]
    Request,
    /// The local server could not start, or is not running.
    #[serde(rename = "E_SERVER")]
    Server,
    /// The answer would exceed the command's documented output size cap.
    #[serde(rename = "E_OUTPUT_CAP")]
    OutputCap,
}

impl ErrorCode {
    /// All codes, in declaration order. Used by tests to pin the public list.
    pub const ALL: &'static [ErrorCode] = &[
        ErrorCode::Stage,
        ErrorCode::Retry,
        ErrorCode::LockHeld,
        ErrorCode::LockNotHeld,
        ErrorCode::PatchStale,
        ErrorCode::PatchInvalid,
        ErrorCode::Schema,
        ErrorCode::HistoryEmpty,
        ErrorCode::Storage,
        ErrorCode::Migration,
        ErrorCode::Auth,
        ErrorCode::NotFound,
        ErrorCode::Request,
        ErrorCode::Server,
        ErrorCode::OutputCap,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            ErrorCode::Stage => "E_STAGE",
            ErrorCode::Retry => "E_RETRY",
            ErrorCode::LockHeld => "E_LOCK_HELD",
            ErrorCode::LockNotHeld => "E_LOCK_NOT_HELD",
            ErrorCode::PatchStale => "E_PATCH_STALE",
            ErrorCode::PatchInvalid => "E_PATCH_INVALID",
            ErrorCode::Schema => "E_SCHEMA",
            ErrorCode::HistoryEmpty => "E_HISTORY_EMPTY",
            ErrorCode::Storage => "E_STORAGE",
            ErrorCode::Migration => "E_MIGRATION",
            ErrorCode::Auth => "E_AUTH",
            ErrorCode::NotFound => "E_NOT_FOUND",
            ErrorCode::Request => "E_REQUEST",
            ErrorCode::Server => "E_SERVER",
            ErrorCode::OutputCap => "E_OUTPUT_CAP",
        }
    }

    /// Default one-line hint telling the caller what to do next.
    pub fn hint(self) -> &'static str {
        match self {
            ErrorCode::Stage => "Run the next allowed step named in the message, then retry.",
            ErrorCode::Retry => "Stop retrying: read the previous error and change the call.",
            ErrorCode::LockHeld => "Wait until the holder releases the layer or the lock expires.",
            ErrorCode::LockNotHeld => "Acquire the lock for the listed layers before patching.",
            ErrorCode::PatchStale => {
                "Re-read only the listed layers, rebuild the patch on the current rev."
            }
            ErrorCode::PatchInvalid => {
                "Fix the patch operations; rev, schemaVersion and locks are read-only."
            }
            ErrorCode::Schema => "Fix the field named in the message so the project validates.",
            ErrorCode::HistoryEmpty => "There is nothing to undo or redo.",
            ErrorCode::Storage => "Check the path, permissions and free disk space, then retry.",
            ErrorCode::Migration => "Upgrade ck: this project was written by a newer version.",
            ErrorCode::Auth => "Use the token from .clipkit/server.json and a localhost origin.",
            ErrorCode::NotFound => "Run ck project get --summary to list the current ids.",
            ErrorCode::Request => "Fix the request body or arguments named in the message.",
            ErrorCode::Server => "Run ck studio --ensure to start the server, then retry.",
            ErrorCode::OutputCap => "Ask for less: read one layer with ck project get --layer.",
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The single error type returned across crate boundaries and serialized to the agent.
#[derive(Debug, Clone, PartialEq, thiserror::Error, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[error("{code}: {message}")]
pub struct CkError {
    pub code: ErrorCode,
    pub message: String,
    pub hint: String,
    /// Layer ids the error is about (for example the changed layers of a stale patch).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layer_ids: Vec<String>,
}

impl CkError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            hint: code.hint().to_owned(),
            layer_ids: Vec::new(),
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = hint.into();
        self
    }

    pub fn with_layers(mut self, layer_ids: impl IntoIterator<Item = String>) -> Self {
        self.layer_ids = layer_ids.into_iter().collect();
        self
    }
}

pub type CkResult<T> = Result<T, CkError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_stable() {
        let table: Vec<(&str, &str)> = ErrorCode::ALL
            .iter()
            .map(|c| (c.as_str(), c.hint()))
            .collect();
        insta::assert_json_snapshot!(table);
    }

    #[test]
    fn display_matches_serde() {
        for code in ErrorCode::ALL {
            assert_eq!(serde_json::to_string(code).unwrap(), format!("\"{code}\""));
            assert!(code.hint().len() <= 100, "{code} hint too long");
        }
    }

    #[test]
    fn error_serializes_compactly() {
        let e = CkError::new(ErrorCode::PatchStale, "base rev 3, current 5")
            .with_layers(["a".to_owned()]);
        let json = serde_json::to_value(&e).unwrap();
        assert_eq!(json["code"], "E_PATCH_STALE");
        assert_eq!(json["layerIds"][0], "a");
        assert_eq!(e.to_string(), "E_PATCH_STALE: base rev 3, current 5");
    }
}

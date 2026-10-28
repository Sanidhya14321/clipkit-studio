//! Project model, JSON patches, revisions, undo, locks, storage and migrations.
//!
//! Pure and synchronous: no async, no network. Every public type is serde + JsonSchema.

pub mod document;
pub mod error;
pub mod history;
pub mod locks;
pub mod log;
pub mod migrate;
pub mod model;
pub mod patch;
pub mod storage;
pub mod timeline;

pub use document::Document;
pub use error::{CkError, CkResult, ErrorCode};
pub use model::Project;

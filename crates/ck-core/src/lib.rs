//! Project model, JSON patches, revisions, undo, locks, storage and migrations.
//!
//! Pure and synchronous: no async, no network. Every public type is serde + JsonSchema.

pub mod error;
pub mod model;

pub use error::{CkError, CkResult, ErrorCode};
pub use model::Project;

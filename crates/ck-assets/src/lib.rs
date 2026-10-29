//! Asset ingestion, cards, catalog, search index, AST extraction and Source Hub.
//!
//! Phase 4 step 1: loaders and validators for the starter taste library and looks (spec 7.1, 7.5).

pub mod color;
pub mod look;
pub mod taste;

use std::path::Path;

use ck_core::{CkError, ErrorCode};

pub use look::{Colors, Easing, Fonts, Look, Motion, MotionPersonality, load_looks};
pub use taste::{TasteFile, load_taste};

/// `E_ASSET` error naming the file and what is wrong with it.
pub(crate) fn asset_error(path: &Path, what: impl std::fmt::Display) -> CkError {
    CkError::new(ErrorCode::Asset, format!("{}: {what}", path.display()))
}

/// Reads one asset file as UTF-8.
pub(crate) fn read_file(path: &Path) -> Result<String, CkError> {
    std::fs::read_to_string(path)
        .map_err(|e| CkError::new(ErrorCode::Storage, format!("{}: {e}", path.display())))
}

/// Files with the given extension in `dir`, sorted by name.
pub(crate) fn list_files(dir: &Path, ext: &str) -> Result<Vec<std::path::PathBuf>, CkError> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| CkError::new(ErrorCode::Storage, format!("{}: {e}", dir.display())))?;
    let mut out = Vec::new();
    for entry in entries {
        let path = entry
            .map_err(|e| CkError::new(ErrorCode::Storage, format!("{}: {e}", dir.display())))?
            .path();
        if path.extension().is_some_and(|x| x == ext) {
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}

/// File stem as UTF-8, used as the expected asset id.
pub(crate) fn stem(path: &Path) -> &str {
    path.file_stem().and_then(|s| s.to_str()).unwrap_or("")
}

#[cfg(test)]
mod tests;

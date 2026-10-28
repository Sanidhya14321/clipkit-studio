//! On-disk storage under `.clipkit/` (spec 3.3).
//!
//! ```text
//! .clipkit/project.json          current project, written atomically
//! .clipkit/history/base.json     the project at its first revision (replay starting point)
//! .clipkit/history/00000001.json one file per revision
//! ```
//! Atomic write = write a temp file in the same folder, fsync it, rename over the target,
//! fsync the folder. A crash leaves either the old file or the new one, plus at most a temp
//! file that the next load removes.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::Value;

use crate::document::Document;
use crate::error::{CkError, CkResult, ErrorCode};
use crate::history::Revision;
use crate::migrate::migrate;
use crate::model::Project;
use crate::patch::content;

const TMP_MARKER: &str = ".tmp-";

pub struct Store {
    root: PathBuf,
}

fn io_err(path: &Path, e: impl std::fmt::Display) -> CkError {
    CkError::new(ErrorCode::Storage, format!("{}: {e}", path.display()))
}

impl Store {
    /// Use `root` (normally `<workdir>/.clipkit`) as the store folder.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn project_path(&self) -> PathBuf {
        self.root.join("project.json")
    }

    fn history_dir(&self) -> PathBuf {
        self.root.join("history")
    }

    /// Create the folders and write a new project plus its replay base. Refuses to overwrite.
    pub fn init(&self, project: &Project) -> CkResult<()> {
        if self.project_path().exists() {
            return Err(io_err(
                &self.project_path(),
                "a project already exists here",
            ));
        }
        fs::create_dir_all(self.history_dir()).map_err(|e| io_err(&self.history_dir(), e))?;
        write_json(&self.history_dir().join("base.json"), project)?;
        write_json(&self.project_path(), project)
    }

    /// Write every revision not yet on disk, then the project. Revisions go first so a crash
    /// between the two leaves history ahead of the project, which `load` detects.
    pub fn save(&self, doc: &Document) -> CkResult<()> {
        for rev in doc.history() {
            let path = self.revision_path(rev.rev);
            if !path.exists() {
                write_json(&path, rev)?;
            }
        }
        write_json(&self.project_path(), doc.project())
    }

    /// Load the project (migrating it if needed) and its history. Removes leftover temp files.
    pub fn load(&self) -> CkResult<Document> {
        self.remove_temp_files()?;
        let project = migrate(read_json(&self.project_path())?)?;
        let history = self.read_history()?;
        if let Some(last) = history.last()
            && last.rev != project.rev
        {
            return Err(io_err(
                &self.project_path(),
                format!(
                    "project is at rev {} but history ends at rev {}",
                    project.rev, last.rev
                ),
            )
            .with_hint("Run ck replay to rebuild the project from history."));
        }
        Ok(Document::from_parts(project, history))
    }

    /// Rebuild the content from `base.json` plus every forward patch and compare it with the
    /// stored project. Proves the history alone reproduces the project.
    pub fn verify_replay(&self) -> CkResult<()> {
        let base_path = self.history_dir().join("base.json");
        let mut doc = serde_json::to_value(migrate(read_json(&base_path)?)?)
            .map_err(|e| io_err(&base_path, e))?;
        for rev in self.read_history()? {
            json_patch::patch(&mut doc, &rev.forward)
                .map_err(|e| io_err(&self.revision_path(rev.rev), format!("replay failed: {e}")))?;
        }
        let stored = serde_json::to_value(migrate(read_json(&self.project_path())?)?)
            .map_err(|e| io_err(&self.project_path(), e))?;
        if content(&doc) == content(&stored) {
            Ok(())
        } else {
            Err(io_err(
                &self.project_path(),
                "replaying history does not reproduce the project",
            ))
        }
    }

    fn revision_path(&self, rev: u64) -> PathBuf {
        self.history_dir().join(format!("{rev:08}.json"))
    }

    fn read_history(&self) -> CkResult<Vec<Revision>> {
        let dir = self.history_dir();
        let mut revs = Vec::new();
        for entry in fs::read_dir(&dir).map_err(|e| io_err(&dir, e))? {
            let path = entry.map_err(|e| io_err(&dir, e))?.path();
            let is_rev = path
                .file_stem()
                .and_then(|s| s.to_str())
                .is_some_and(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
                && path.extension().is_some_and(|e| e == "json");
            if is_rev {
                let rev: Revision = serde_json::from_value(read_json(&path)?)
                    .map_err(|e| io_err(&path, format!("not a revision: {e}")))?;
                revs.push(rev);
            }
        }
        revs.sort_by_key(|r| r.rev);
        for (i, r) in revs.iter().enumerate() {
            if i > 0 && r.rev != revs[i - 1].rev + 1 {
                return Err(io_err(
                    &dir,
                    format!("history has a gap before rev {}", r.rev),
                ));
            }
        }
        Ok(revs)
    }

    fn remove_temp_files(&self) -> CkResult<()> {
        for dir in [self.root.clone(), self.history_dir()] {
            let Ok(entries) = fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.contains(TMP_MARKER))
                {
                    fs::remove_file(&path).map_err(|e| io_err(&path, e))?;
                }
            }
        }
        Ok(())
    }
}

fn read_json(path: &Path) -> CkResult<Value> {
    let text = fs::read_to_string(path).map_err(|e| io_err(path, e))?;
    serde_json::from_str(&text).map_err(|e| io_err(path, format!("invalid JSON: {e}")))
}

fn write_json(path: &Path, value: &impl Serialize) -> CkResult<()> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|e| io_err(path, e))?;
    bytes.push(b'\n');
    atomic_write(path, &bytes)
}

/// Write `bytes` to `path` so readers see either the old content or the new, never a mix.
pub fn atomic_write(path: &Path, bytes: &[u8]) -> CkResult<()> {
    let dir = path
        .parent()
        .ok_or_else(|| io_err(path, "path has no parent folder"))?;
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
    let tmp = dir.join(format!(".{name}{TMP_MARKER}{}", std::process::id()));
    let result = (|| {
        let mut f = File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        fs::rename(&tmp, path)?;
        sync_dir(dir)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp); // best effort; load() also removes leftovers
    }
    result.map_err(|e| io_err(path, e))
}

#[cfg(unix)]
fn sync_dir(dir: &Path) -> std::io::Result<()> {
    File::open(dir)?.sync_all()
}

#[cfg(not(unix))]
fn sync_dir(_dir: &Path) -> std::io::Result<()> {
    Ok(()) // directories cannot be opened for fsync on Windows; rename is still atomic.
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::model::Author;
    use crate::patch::PatchRequest;

    fn edited_doc() -> Document {
        let mut doc = Document::new(crate::model::tests::sample());
        for (i, at) in [1.0, 2.0, 3.5].iter().enumerate() {
            let req = PatchRequest {
                base_rev: doc.project().rev,
                layer_revs: Default::default(),
                ops: serde_json::from_value(
                    json!([{"op": "replace", "path": "/tracks/1/items/0/at", "value": at}]),
                )
                .unwrap(),
                author: Author::Human,
                label: format!("move {i}"),
            };
            doc.patch(&req, i as u64).unwrap();
        }
        doc.undo(Author::Human, 10).unwrap();
        doc
    }

    #[test]
    fn save_load_round_trip_and_replay() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path().join(".clipkit"));
        store.init(&crate::model::tests::sample()).unwrap();
        let doc = edited_doc();
        store.save(&doc).unwrap();
        let loaded = store.load().unwrap();
        assert_eq!(loaded, doc);
        store.verify_replay().unwrap();
        assert_eq!(
            fs::read_dir(dir.path().join(".clipkit/history"))
                .unwrap()
                .count(),
            5
        ); // base + 4
        assert!(
            store.init(doc.project()).is_err(),
            "init must not overwrite"
        );
    }

    #[test]
    fn leftover_temp_files_are_removed_and_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path());
        store.init(&crate::model::tests::sample()).unwrap();
        // Simulate a crash mid-write: a half-written temp file next to the real one.
        let tmp = dir.path().join(".project.json.tmp-999");
        fs::write(&tmp, b"{\"half").unwrap();
        store.load().unwrap();
        assert!(!tmp.exists());
    }

    #[test]
    fn corruption_is_reported_with_storage_code() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path());
        store.init(&crate::model::tests::sample()).unwrap();
        store.save(&edited_doc()).unwrap();

        // Tampered project content: load works, replay catches it.
        let mut p: Value = read_json(&store.project_path()).unwrap();
        p["name"] = json!("tampered");
        write_json(&store.project_path(), &p).unwrap();
        assert_eq!(store.verify_replay().unwrap_err().code, ErrorCode::Storage);

        // Corrupt history file.
        fs::write(dir.path().join("history/00000002.json"), b"not json").unwrap();
        assert_eq!(store.load().unwrap_err().code, ErrorCode::Storage);

        // Missing history file leaves a gap.
        fs::remove_file(dir.path().join("history/00000002.json")).unwrap();
        let err = store.load().unwrap_err();
        assert!(err.message.contains("gap"), "{err}");
    }

    #[test]
    fn project_behind_history_is_detected() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path());
        let base = crate::model::tests::sample();
        store.init(&base).unwrap();
        let doc = edited_doc();
        // Crash after revisions were written but before project.json: history is ahead.
        for rev in doc.history() {
            write_json(&store.revision_path(rev.rev), rev).unwrap();
        }
        let err = store.load().unwrap_err();
        assert_eq!(err.code, ErrorCode::Storage);
        assert!(err.hint.contains("replay"));
    }
}

//! `.clipkit/server.json`: how clients find the running server (spec 5.1).

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use ck_core::{CkError, CkResult, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Token length in random bytes (hex-encoded to twice this many characters).
const TOKEN_BYTES: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    pub port: u16,
    /// Session token; every API and WebSocket request must carry it.
    pub token: String,
    pub pid: u32,
    /// Studio URL without the token.
    pub url: String,
}

impl ServerInfo {
    pub fn new(port: u16, token: String) -> Self {
        Self {
            port,
            token,
            pid: std::process::id(),
            url: format!("http://127.0.0.1:{port}/"),
        }
    }

    pub fn path(root: &Path) -> PathBuf {
        root.join("server.json")
    }

    /// Write atomically. On unix the temp file is created 0600 before the token goes in, so the
    /// token is never readable by other users, not even briefly.
    pub fn write(&self, root: &Path) -> CkResult<()> {
        let path = Self::path(root);
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| server_err(&path, e))?;
        let tmp = root.join(format!(".server.json.tmp-{}", std::process::id()));
        let _ = std::fs::remove_file(&tmp); // a leftover could carry wider permissions
        let result = (|| {
            let mut f = private_file(&tmp)?;
            f.write_all(&bytes)?;
            f.sync_all()?;
            std::fs::rename(&tmp, &path)
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&tmp);
        }
        result.map_err(|e| server_err(&path, e))
    }

    pub fn read(root: &Path) -> CkResult<Self> {
        let path = Self::path(root);
        let bytes = std::fs::read(&path).map_err(|e| server_err(&path, e))?;
        serde_json::from_slice(&bytes).map_err(|e| server_err(&path, e))
    }

    /// Remove the file if it still describes this server (another server may have replaced it).
    pub fn remove_if_ours(&self, root: &Path) {
        if Self::read(root).is_ok_and(|on_disk| on_disk == *self) {
            let _ = std::fs::remove_file(Self::path(root));
        }
    }
}

/// A fresh random session token from the OS generator.
pub fn new_token() -> CkResult<String> {
    let mut bytes = [0u8; TOKEN_BYTES];
    getrandom::fill(&mut bytes)
        .map_err(|e| CkError::new(ErrorCode::Server, format!("no OS randomness: {e}")))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

fn server_err(path: &Path, e: impl std::fmt::Display) -> CkError {
    CkError::new(ErrorCode::Server, format!("{}: {e}", path.display()))
}

#[cfg(unix)]
fn private_file(path: &Path) -> std::io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
}

#[cfg(not(unix))]
fn private_file(path: &Path) -> std::io::Result<File> {
    OpenOptions::new().write(true).create_new(true).open(path)
}

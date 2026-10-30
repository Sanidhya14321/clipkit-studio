//! `ck sources vet` (spec 7.6): fetch a source into quarantine and vet it without running any of
//! its code. The fetch is a shallow git clone with hooks, submodules and LFS filters off (or a
//! copy of a local folder); files are made read-only. Vetting reads the license text, looks
//! for malware lures and reviews dependencies statically, then pins the result in
//! `sources.lock`. A pass marks the source vetted and enabled; any blocker rejects it; an
//! unrecognized license leaves it unverified.

use std::path::{Path, PathBuf};
use std::process::Command;

use ck_core::{CkError, CkResult, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::card::LicenseClass;
use crate::hub::{Finding, Hub, LockEntry, Severity, SourceStatus};
use crate::license::classify;

/// Largest single file copied into quarantine.
pub const MAX_FILE: u64 = 2 * 1024 * 1024;
/// Audio has a separate cap; the source-wide 64 MiB cap still applies.
pub const MAX_AUDIO_FILE: u64 = 32 * 1024 * 1024;

pub(crate) fn file_limit(path: &Path) -> u64 {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("wav" | "mp3" | "ogg" | "opus" | "flac" | "m4a") => MAX_AUDIO_FILE,
        _ => MAX_FILE,
    }
}
/// Largest total size of one source.
pub const MAX_TOTAL: u64 = 64 * 1024 * 1024;
/// Most files in one source.
pub const MAX_FILES: usize = 5000;

pub mod caps {
    pub const VET: usize = 4 * 1024;
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VetRequest {
    pub id: String,
    /// Vet a local folder instead of cloning the repo (fixtures, manual downloads).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VetResult {
    pub id: String,
    pub status: SourceStatus,
    pub enabled: bool,
    pub license_class: LicenseClass,
    pub spdx: String,
    pub commit: String,
    pub files: usize,
    pub findings: Vec<Finding>,
    #[serde(default, skip_serializing_if = "crate::sources::is_zero")]
    pub omitted: usize,
}

fn io(path: &Path, e: impl std::fmt::Display) -> CkError {
    CkError::new(ErrorCode::Storage, format!("{}: {e}", path.display()))
}

fn finding(severity: Severity, rule: &str, path: &str, detail: impl Into<String>) -> Finding {
    Finding {
        severity,
        rule: rule.into(),
        path: path.into(),
        detail: detail.into(),
    }
}

/// Removes a quarantine folder whose files were made read-only.
fn clear(dir: &Path) -> CkResult<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in walk(dir)? {
        if let Ok(meta) = std::fs::symlink_metadata(&entry) {
            let mut p = meta.permissions();
            #[allow(clippy::permissions_set_readonly_false)]
            p.set_readonly(false);
            let _ = std::fs::set_permissions(&entry, p);
        }
    }
    std::fs::remove_dir_all(dir).map_err(|e| io(dir, e))
}

/// Every path under `dir` (files and folders), depth first, symlinks not followed.
fn walk(dir: &Path) -> CkResult<Vec<PathBuf>> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).map_err(|e| io(&d, e))? {
            let path = e.map_err(|e| io(&d, e))?.path();
            let meta = std::fs::symlink_metadata(&path).map_err(|e| io(&path, e))?;
            if meta.is_dir() {
                stack.push(path.clone());
            }
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}

/// Copies the regular files of `from` into `to`; symlinks and special files are skipped and
/// reported. Enforces the file count and size limits.
fn copy_local(from: &Path, to: &Path, findings: &mut Vec<Finding>) -> CkResult<()> {
    let mut total = 0;
    let mut count = 0;
    for path in walk(from)? {
        let rel = path.strip_prefix(from).map_err(|e| io(&path, e))?;
        if rel.components().any(|c| c.as_os_str() == ".git") {
            continue;
        }
        let meta = std::fs::symlink_metadata(&path).map_err(|e| io(&path, e))?;
        let shown = rel.to_string_lossy().replace('\\', "/");
        if meta.file_type().is_symlink() {
            findings.push(finding(Severity::Warning, "symlink", &shown, "skipped"));
            continue;
        }
        let dest = to.join(rel);
        if meta.is_dir() {
            std::fs::create_dir_all(&dest).map_err(|e| io(&dest, e))?;
            continue;
        }
        if !meta.is_file() {
            continue;
        }
        count += 1;
        total += meta.len();
        if meta.len() > file_limit(&path) {
            findings.push(finding(
                Severity::Warning,
                "large-file",
                &shown,
                format!("{} bytes, not copied", meta.len()),
            ));
            continue;
        }
        if count > MAX_FILES || total > MAX_TOTAL {
            return Err(CkError::new(
                ErrorCode::Request,
                format!("source is over {MAX_FILES} files or {MAX_TOTAL} bytes"),
            ));
        }
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| io(parent, e))?;
        }
        std::fs::copy(&path, &dest).map_err(|e| io(&path, e))?;
    }
    Ok(())
}

/// Shallow clone of `owner/name` from GitHub into `to`, running none of its code: hooks off,
/// no submodules, LFS smudge off, no terminal prompts. Returns the commit.
fn git_fetch(repo: &str, to: &Path) -> CkResult<String> {
    let url = format!("https://github.com/{repo}.git");
    let git_err = |what: &str, out: &std::process::Output| {
        CkError::new(
            ErrorCode::Request,
            format!(
                "git {what} failed: {}",
                String::from_utf8_lossy(&out.stderr)
                    .lines()
                    .next()
                    .unwrap_or("")
            ),
        )
        .with_hint("Check the repo name and network, or vet a local copy with --from.")
    };
    let out = Command::new("git")
        .args([
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "protocol.file.allow=never",
            "-c",
            "filter.lfs.smudge=",
            "-c",
            "filter.lfs.process=",
            "-c",
            "filter.lfs.required=false",
            "clone",
            "--depth",
            "1",
            "--single-branch",
            "--no-tags",
            "--quiet",
            "--",
            &url,
        ])
        .arg(to)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_LFS_SKIP_SMUDGE", "1")
        .output()
        .map_err(|e| CkError::new(ErrorCode::Request, format!("git could not start: {e}")))?;
    if !out.status.success() {
        return Err(git_err("clone", &out));
    }
    let head = Command::new("git")
        .arg("-C")
        .arg(to)
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|e| CkError::new(ErrorCode::Request, format!("git could not start: {e}")))?;
    if !head.status.success() {
        return Err(git_err("rev-parse", &head));
    }
    let commit = String::from_utf8_lossy(&head.stdout).trim().to_owned();
    let git_dir = to.join(".git");
    std::fs::remove_dir_all(&git_dir).map_err(|e| io(&git_dir, e))?;
    Ok(commit)
}

/// SHA-256 over every file path and content: the "commit" of a local copy.
fn tree_hash(dir: &Path) -> CkResult<String> {
    let mut h = Sha256::new();
    for path in walk(dir)? {
        if path.is_file() {
            let rel = path.strip_prefix(dir).map_err(|e| io(&path, e))?;
            h.update(rel.to_string_lossy().as_bytes());
            h.update(std::fs::read(&path).map_err(|e| io(&path, e))?);
        }
    }
    Ok(hex(&h.finalize()))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

const BINARY_EXT: &[&str] = &[
    "exe", "dll", "so", "dylib", "bin", "msi", "dmg", "jar", "node",
];
const CODE_EXT: &[&str] = &[
    "js", "jsx", "ts", "tsx", "mjs", "cjs", "glsl", "frag", "vert",
];
const INSTALL_SCRIPTS: &[&str] = &[
    "preinstall",
    "install",
    "postinstall",
    "prepare",
    "prepublish",
];

/// Static checks over the quarantined files.
pub fn scan(dir: &Path) -> CkResult<Vec<Finding>> {
    let mut out = Vec::new();
    for path in walk(dir)? {
        if !path.is_file() {
            continue;
        }
        let rel = path
            .strip_prefix(dir)
            .map_err(|e| io(&path, e))?
            .to_string_lossy()
            .replace('\\', "/");
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let bytes = std::fs::read(&path).map_err(|e| io(&path, e))?;
        let magic = bytes.starts_with(b"\x7fELF")
            || bytes.starts_with(b"MZ")
            || bytes.starts_with(&[0xcf, 0xfa, 0xed, 0xfe])
            || bytes.starts_with(&[0xca, 0xfe, 0xba, 0xbe]);
        if magic || BINARY_EXT.contains(&ext.as_str()) {
            out.push(finding(
                Severity::Blocker,
                "binary-blob",
                &rel,
                "executable or binary file",
            ));
            continue;
        }
        let Ok(text) = String::from_utf8(bytes) else {
            continue;
        };
        let lower = text.to_lowercase();
        if rel.ends_with("package.json") {
            scan_package(&rel, &text, &mut out);
        }
        if CODE_EXT.contains(&ext.as_str()) {
            for (needle, what) in [
                ("eval(", "eval"),
                ("new function(", "new Function"),
                ("atob(", "base64 decode"),
                ("'base64')", "base64 decode"),
                ("\"base64\")", "base64 decode"),
            ] {
                if lower.contains(needle) {
                    out.push(finding(Severity::Blocker, "dynamic-code", &rel, what));
                }
            }
            // Build tooling often spawns processes; it is reported, never run.
            if lower.contains("child_process") {
                out.push(finding(
                    Severity::Warning,
                    "spawns-processes",
                    &rel,
                    "child_process",
                ));
            }
            for (needle, what) in [("math.random(", "Math.random"), ("date.now(", "Date.now")] {
                if lower.contains(needle) {
                    out.push(finding(
                        Severity::Warning,
                        "nondeterministic",
                        &rel,
                        format!("{what}: frames may differ between renders"),
                    ));
                }
            }
        }
        if ext == "md" || ext == "txt" || ext == "sh" || rel.to_lowercase().starts_with("readme") {
            scan_text(&rel, &lower, &mut out);
        }
    }
    Ok(out)
}

fn scan_package(rel: &str, text: &str, out: &mut Vec<Finding>) {
    let Ok(pkg) = serde_json::from_str::<serde_json::Value>(text) else {
        out.push(finding(
            Severity::Warning,
            "manifest",
            rel,
            "package.json does not parse",
        ));
        return;
    };
    if let Some(scripts) = pkg.get("scripts").and_then(|s| s.as_object()) {
        for name in INSTALL_SCRIPTS {
            let Some(body) = scripts.get(*name).and_then(|b| b.as_str()) else {
                continue;
            };
            // Spec 7.6 names obfuscated install scripts as the lure; a plain one is reported
            // (ck never runs it either way).
            let lower = body.to_lowercase();
            let obfuscated = ["eval", "base64", "curl", "wget", "http", "\\x", "atob"]
                .iter()
                .any(|m| lower.contains(m));
            let severity = if obfuscated {
                Severity::Blocker
            } else {
                Severity::Warning
            };
            out.push(finding(
                severity,
                "install-script",
                rel,
                format!("`{name}`: {body}"),
            ));
        }
    }
    for table in [
        "dependencies",
        "devDependencies",
        "peerDependencies",
        "optionalDependencies",
    ] {
        let Some(deps) = pkg.get(table).and_then(|d| d.as_object()) else {
            continue;
        };
        for (name, spec) in deps {
            let spec = spec.as_str().unwrap_or("");
            let remote = ["git", "http:", "https:", "file:", "link:", "github:"]
                .iter()
                .any(|p| spec.starts_with(p))
                || spec.contains("://");
            if remote {
                out.push(finding(
                    Severity::Blocker,
                    "non-registry-dependency",
                    rel,
                    format!("{name} from {spec}"),
                ));
            }
        }
    }
}

fn scan_text(rel: &str, lower: &str, out: &mut Vec<Finding>) {
    let pipe_shell = [
        "| sh",
        "|sh",
        "| bash",
        "|bash",
        "iex(",
        "invoke-expression",
    ];
    if (lower.contains("curl ") || lower.contains("wget ") || lower.contains("iwr "))
        && pipe_shell.iter().any(|p| lower.contains(p))
    {
        out.push(finding(
            Severity::Blocker,
            "download-lure",
            rel,
            "download piped into a shell",
        ));
    }
    for word in lower.split_whitespace() {
        let link = word.trim_matches(|c: char| "()[]<>\"'`.,".contains(c));
        let archive = [
            ".zip", ".tar.gz", ".tgz", ".exe", ".msi", ".dmg", ".rar", ".7z",
        ]
        .iter()
        .any(|e| link.ends_with(e));
        if link.starts_with("http") && archive && !link.starts_with("https://github.com/") {
            out.push(finding(
                Severity::Blocker,
                "download-lure",
                rel,
                format!("download from a non-GitHub host: {link}"),
            ));
        }
    }
    let injection = [
        "ignore all previous instructions",
        "ignore previous instructions",
        "to ai agents",
        "ai agent:",
        "skip vetting",
    ];
    if injection.iter().any(|p| lower.contains(p)) {
        out.push(finding(
            Severity::Warning,
            "prompt-injection",
            rel,
            "text addressed to AI agents (treated as data)",
        ));
    }
}

fn license_text(dir: &Path) -> CkResult<Option<(String, String)>> {
    for e in std::fs::read_dir(dir).map_err(|e| io(dir, e))? {
        let path = e.map_err(|e| io(dir, e))?.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_uppercase();
        if path.is_file()
            && ["LICENSE", "LICENCE", "COPYING", "UNLICENSE"]
                .iter()
                .any(|p| name.starts_with(p))
        {
            let text = std::fs::read_to_string(&path).map_err(|e| io(&path, e))?;
            return Ok(Some((name, text)));
        }
    }
    Ok(None)
}

fn make_read_only(dir: &Path) -> CkResult<()> {
    for path in walk(dir)? {
        if path.is_file() {
            let mut p = std::fs::metadata(&path)
                .map_err(|e| io(&path, e))?
                .permissions();
            p.set_readonly(true);
            std::fs::set_permissions(&path, p).map_err(|e| io(&path, e))?;
        }
    }
    Ok(())
}

fn today() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let days = (secs / 86_400) as i64;
    // Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

pub fn vet(hub: &Hub, req: &VetRequest) -> CkResult<VetResult> {
    let mut source = hub.source(&req.id)?;
    let dest = hub.quarantine().join(&source.id);
    clear(&dest)?;
    std::fs::create_dir_all(hub.quarantine()).map_err(|e| io(&hub.quarantine(), e))?;
    let mut findings = Vec::new();
    let commit = match (&req.from, &source.repo) {
        (Some(from), _) => {
            let from = PathBuf::from(from);
            if !from.is_dir() {
                return Err(CkError::new(
                    ErrorCode::NotFound,
                    format!("no folder {}", from.display()),
                ));
            }
            std::fs::create_dir_all(&dest).map_err(|e| io(&dest, e))?;
            if let Err(e) = copy_local(&from, &dest, &mut findings) {
                clear(&dest)?;
                return Err(e);
            }
            format!("local-{}", &tree_hash(&dest)?[..16])
        }
        (None, Some(repo)) => {
            let commit = git_fetch(repo, &dest)?;
            // Same limits as a local copy: drop large files, report symlinks.
            let staged = hub.quarantine().join(format!(".{}-staged", source.id));
            clear(&staged)?;
            std::fs::rename(&dest, &staged).map_err(|e| io(&dest, e))?;
            std::fs::create_dir_all(&dest).map_err(|e| io(&dest, e))?;
            let copied = copy_local(&staged, &dest, &mut findings);
            clear(&staged)?;
            if copied.is_err() {
                clear(&dest)?;
            }
            copied?;
            commit
        }
        (None, None) => {
            return Err(CkError::new(
                ErrorCode::Request,
                format!("source {:?} has no repo", source.id),
            )
            .with_hint(
                "Resolve the repo from the official listing, or vet a local copy with --from.",
            ));
        }
    };
    findings.extend(scan(&dest)?);
    let (license_hash, reading) = match license_text(&dest)? {
        Some((_, text)) => (hex(&Sha256::digest(text.as_bytes())), classify(&text)),
        None => {
            findings.push(finding(
                Severity::Warning,
                "no-license",
                "",
                "no LICENSE file",
            ));
            (String::new(), classify(""))
        }
    };
    make_read_only(&dest)?;
    let files = walk(&dest)?.iter().filter(|p| p.is_file()).count();
    let blocked = findings.iter().any(|f| f.severity == Severity::Blocker);
    let status = if blocked {
        SourceStatus::Rejected
    } else if reading.class == LicenseClass::Unknown {
        SourceStatus::Unverified
    } else {
        SourceStatus::Vetted
    };
    findings.sort_by(|a, b| b.severity.cmp(&a.severity).then(a.path.cmp(&b.path)));
    source.status = status;
    source.enabled = status == SourceStatus::Vetted;
    source.license_class = reading.class;
    if status == SourceStatus::Vetted {
        source.license = if reading.spdx.is_empty() {
            "custom terms (see LICENSE)".into()
        } else {
            reading.spdx.clone()
        };
    }
    hub.put_source(source.clone())?;
    let mut lock = hub.lock()?;
    lock.insert(
        source.id.clone(),
        LockEntry {
            commit: commit.clone(),
            license_hash,
            license_class: reading.class,
            spdx: reading.spdx.clone(),
            vetted_at: today(),
            findings: findings.clone(),
        },
    );
    hub.save_lock(&lock)?;
    Ok(VetResult {
        id: source.id,
        status,
        enabled: source.enabled,
        license_class: reading.class,
        spdx: reading.spdx,
        commit,
        files,
        findings,
        omitted: 0,
    })
}

#[cfg(test)]
mod tests;

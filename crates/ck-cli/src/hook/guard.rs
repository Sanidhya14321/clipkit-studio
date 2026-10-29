//! `ck hook guard` rules (spec 6.1): direct edits to .clipkit/, assets/ and media files, and shell
//! commands that bypass ck, are refused with a reason naming the ck way instead. Shell commands
//! are read word by word, a heuristic: the server enforces every guardrail anyway.

use std::path::{Component, Path, PathBuf};

pub const CLIPKIT: &str = "ClipKit: .clipkit/ holds the live project and its token. Read it with \
     ck status or ck project get; change it with ck lock, then ck project patch.";
pub const ASSETS: &str =
    "ClipKit: assets/ is the knowledge base and is managed by ck; do not change it directly.";
pub const MEDIA: &str = "ClipKit: media files are sources and are never edited. Cuts, mixes and \
     output go through ck (ck project patch, ck export).";
pub const MEDIA_TOOL: &str = "ClipKit: do not run ffmpeg, ffprobe or sox on project media; ck \
     runs them. Edit with ck project patch and render with ck export.";

const MEDIA_EXTENSIONS: &[&str] = &[
    "mp4", "mov", "m4v", "mkv", "webm", "avi", "mxf", "mts", "wav", "mp3", "m4a", "aac", "flac",
    "ogg", "opus", "aif", "aiff", "jpg", "jpeg", "png", "gif", "webp", "heic", "tif", "tiff",
];
const MEDIA_TOOLS: &[&str] = &["ffmpeg", "ffprobe", "ffplay", "sox"];
/// Commands that run the next word as the program.
const WRAPPERS: &[&str] = &[
    "sudo", "env", "time", "nice", "nohup", "exec", "command", "xargs", "timeout", "stdbuf",
];
/// Commands that change every path they are given.
const MUTATORS: &[&str] = &[
    "rm", "rmdir", "mv", "tee", "touch", "mkdir", "chmod", "chown", "truncate", "dd", "unlink",
    "shred",
];
/// Commands that change only their last path (the destination).
const COPIERS: &[&str] = &["cp", "rsync", "install", "ln"];

/// File tools (Write, Edit, MultiEdit, NotebookEdit): the reason to refuse writing `path`, if any.
/// `cwd` resolves a relative path; `dir` is the project folder.
pub fn file(path: &Path, cwd: &Path, dir: &Path) -> Option<&'static str> {
    let path = normalize(&cwd.join(path));
    if path
        .components()
        .any(|c| c.as_os_str() == std::ffi::OsStr::new(".clipkit"))
    {
        return Some(CLIPKIT);
    }
    if path.starts_with(normalize(&dir.join("assets"))) {
        return Some(ASSETS);
    }
    is_media(&path.to_string_lossy()).then_some(MEDIA)
}

/// Bash: the reason to refuse `command`, if any.
pub fn bash(command: &str) -> Option<&'static str> {
    let command = command.replace("&>", ">").replace(">&", ">");
    command
        .split([';', '|', '&', '\n', '(', ')', '`', '{', '}'])
        .find_map(segment)
}

fn segment(segment: &str) -> Option<&'static str> {
    let words: Vec<String> = segment
        .split_whitespace()
        .map(|w| w.replace(['\'', '"'], ""))
        .filter(|w| !w.is_empty())
        .collect();
    if words.iter().any(|w| w.contains(".clipkit")) {
        return Some(CLIPKIT);
    }
    let start = words
        .iter()
        .position(|w| !is_prefix_word(w))
        .unwrap_or(words.len());
    let program = words.get(start).map(|w| basename(w)).unwrap_or_default();
    if MEDIA_TOOLS.contains(&program) {
        return Some(MEDIA_TOOL);
    }
    let args: Vec<&str> = words
        .iter()
        .skip(start + 1)
        .map(String::as_str)
        .filter(|w| !w.starts_with('-') && !w.contains('>'))
        .collect();
    let in_place = words.iter().any(|w| w == "-i" || w.starts_with("-i."));
    let mut targets: Vec<&str> =
        if MUTATORS.contains(&program) || (in_place && matches!(program, "sed" | "perl")) {
            args
        } else if COPIERS.contains(&program) {
            args.last().copied().into_iter().collect()
        } else {
            Vec::new()
        };
    targets.extend(redirect_targets(&words));
    targets.into_iter().find_map(|t| {
        if is_assets(t) {
            Some(ASSETS)
        } else {
            is_media(t).then_some(MEDIA)
        }
    })
}

/// The paths after `>` or `>>` (`2>` and the like included).
fn redirect_targets(words: &[String]) -> Vec<&str> {
    let mut out = Vec::new();
    for (i, w) in words.iter().enumerate() {
        if let Some(pos) = w.rfind('>') {
            let rest = &w[pos + 1..];
            if !rest.is_empty() {
                out.push(rest);
            } else if let Some(next) = words.get(i + 1) {
                out.push(next.as_str());
            }
        }
    }
    out
}

/// Environment assignments, wrappers, their flags and numbers before the real program.
fn is_prefix_word(w: &str) -> bool {
    (w.contains('=') && !w.starts_with('-'))
        || WRAPPERS.contains(&basename(w))
        || w.starts_with('-')
        || w.parse::<f64>().is_ok()
}

fn basename(w: &str) -> &str {
    w.rsplit(['/', '\\']).next().unwrap_or(w)
}

fn is_assets(w: &str) -> bool {
    let w = w.strip_prefix("./").unwrap_or(w);
    w == "assets" || w.starts_with("assets/") || w.contains("/assets/") || w.ends_with("/assets")
}

fn is_media(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| MEDIA_EXTENSIONS.contains(&e.to_lowercase().as_str()))
}

/// `a/./b/../c` to `a/c` without touching the file system (the file may not exist yet).
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIR: &str = "/work/proj";

    fn file_reason(path: &str) -> Option<&'static str> {
        file(Path::new(path), Path::new(DIR), Path::new(DIR))
    }

    #[test]
    fn file_tools_are_refused_on_project_state_assets_and_media() {
        assert_eq!(file_reason(".clipkit/project.json"), Some(CLIPKIT));
        assert_eq!(
            file_reason("/work/proj/.clipkit/server.json"),
            Some(CLIPKIT)
        );
        assert_eq!(file_reason("src/../.clipkit/x"), Some(CLIPKIT));
        assert_eq!(file_reason("assets/cards/a.json"), Some(ASSETS));
        assert_eq!(
            file_reason("/work/proj/assets/quarantine/r.md"),
            Some(ASSETS)
        );
        assert_eq!(file_reason("footage/take1.MP4"), Some(MEDIA));
        assert_eq!(file_reason("music/bed.wav"), Some(MEDIA));
    }

    #[test]
    fn file_tools_are_allowed_elsewhere() {
        assert_eq!(file_reason("src/main.rs"), None);
        assert_eq!(file_reason("README.md"), None);
        assert_eq!(file_reason("docs/assets.md"), None);
        assert_eq!(file_reason("/other/assets/x.json"), None);
        assert_eq!(file_reason("logo.svg"), None);
    }

    #[test]
    fn shell_commands_that_bypass_ck_are_refused() {
        let cases = [
            ("ffmpeg -i footage/a.mp4 -t 3 out.mp4", MEDIA_TOOL),
            ("cd footage && /usr/bin/ffprobe a.mov", MEDIA_TOOL),
            ("FOO=1 sudo ffmpeg -version", MEDIA_TOOL),
            ("timeout 5 sox in.wav out.wav", MEDIA_TOOL),
            ("cat .clipkit/project.json", CLIPKIT),
            ("jq . './.clipkit/server.json'", CLIPKIT),
            ("rm -rf assets/cards", ASSETS),
            ("echo x > assets/new.json", ASSETS),
            ("echo x >>./assets/new.json", ASSETS),
            ("sed -i s/a/b/ assets/cards/a.json", ASSETS),
            ("cp notes.md assets/", ASSETS),
            ("mv footage/a.mp4 footage/b.mp4", MEDIA),
            ("ls; rm music/bed.wav", MEDIA),
            ("$(ffmpeg -i a.mp4)", MEDIA_TOOL),
        ];
        for (cmd, reason) in cases {
            assert_eq!(bash(cmd), Some(reason), "{cmd}");
        }
    }

    #[test]
    fn ordinary_shell_commands_are_allowed() {
        for cmd in [
            "cargo test",
            "ck project patch - < patch.json",
            "ck status",
            "ls assets/cards",
            "cat assets/cards/a.json 2>&1 | head",
            "rg ffmpeg crates/",
            "cp assets/cards/a.json /tmp/a.json",
            "sed s/a/b/ assets/cards/a.json",
            "git diff -- footage.mp4",
            "which ffmpeg",
        ] {
            assert_eq!(bash(cmd), None, "{cmd}");
        }
    }
}

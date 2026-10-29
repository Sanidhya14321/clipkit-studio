//! Runs ffmpeg and ffprobe as argument arrays with a timeout. Never a shell string.

use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use ck_core::{CkError, ErrorCode};

/// Largest stdout or stderr kept from one tool call.
const MAX_OUTPUT: u64 = 8 * 1024 * 1024;

/// Refuses a user path that a tool could read as an option.
pub(crate) fn check_path(path: &Path) -> Result<(), CkError> {
    let s = path.to_string_lossy();
    if s.is_empty() || s.starts_with('-') {
        return Err(CkError::new(
            ErrorCode::Request,
            format!("media path {s:?} is empty or starts with '-'"),
        ));
    }
    Ok(())
}

/// Output of a finished tool call.
pub(crate) struct Output {
    pub stdout: Vec<u8>,
}

/// Runs `program` with `args`, killing it after `timeout`. Fails with `E_MEDIA` on a missing
/// tool, a non-zero exit (first stderr line in the message) or a timeout.
pub(crate) fn run(
    program: &str,
    args: &[&std::ffi::OsStr],
    timeout: Duration,
) -> Result<Output, CkError> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| CkError::new(ErrorCode::Media, format!("{program} could not start: {e}")))?;
    let (out, err) = (child.stdout.take(), child.stderr.take());
    let read_all = |r: Option<Box<dyn Read + Send>>| {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(r) = r {
                let _ = r.take(MAX_OUTPUT).read_to_end(&mut buf);
            }
            buf
        })
    };
    let out_t = read_all(out.map(|o| Box::new(o) as Box<dyn Read + Send>));
    let err_t = read_all(err.map(|e| Box::new(e) as Box<dyn Read + Send>));
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if start.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(CkError::new(
                    ErrorCode::Media,
                    format!("{program} timed out after {} s", timeout.as_secs()),
                ));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            Err(e) => return Err(CkError::new(ErrorCode::Media, format!("{program}: {e}"))),
        }
    };
    let stdout = out_t.join().unwrap_or_default();
    let stderr = err_t.join().unwrap_or_default();
    if !status.success() {
        let line = String::from_utf8_lossy(&stderr);
        let line = line.lines().next().unwrap_or("no error output").trim();
        return Err(CkError::new(
            ErrorCode::Media,
            format!("{program} failed ({status}): {}", truncate(line, 200)),
        ));
    }
    Ok(Output { stdout })
}

fn truncate(s: &str, max: usize) -> &str {
    match s.char_indices().nth(max) {
        Some((i, _)) => &s[..i],
        None => s,
    }
}

//! `ck studio --ensure` (spec 5.1): start the studio in the background if it is not running and
//! open the browser tab once per server. Idempotent and safe to call from concurrent hooks.

use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use ck_core::{CkError, CkResult, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::info::ServerInfo;
use crate::ops::{Ops, capped, caps};

/// How long a freshly spawned server may take to answer before ensure gives up.
const READY_LIMIT: Duration = Duration::from_secs(10);
const POLL: Duration = Duration::from_millis(10);
/// Longest stretch of the server log quoted in an error.
const LOG_TAIL: usize = 300;

/// `studio` (MCP) / `ck studio --ensure` input.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnsureRequest {
    /// Open the studio in the browser, once per server.
    #[serde(default)]
    pub open: bool,
}

/// `ck studio --ensure` output. The token stays in server.json.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Ensured {
    pub url: String,
    pub port: u16,
    /// This call started the server.
    pub started: bool,
    /// This call opened the browser tab.
    pub opened: bool,
}

/// How a missing server is started.
#[derive(Debug, Clone)]
pub struct Launcher {
    /// The ck binary to run as `ck studio`.
    pub exe: PathBuf,
    /// Port for the new server; 0 picks a free one.
    pub port: u16,
    /// Idle limit for the new server in seconds.
    pub idle_secs: u64,
}

impl Launcher {
    /// This process's own binary with a free port and the given idle limit.
    pub fn current(idle_secs: u64) -> CkResult<Self> {
        let exe = std::env::current_exe()
            .map_err(|e| server_err(format!("cannot find the ck binary: {e}")))?;
        Ok(Self {
            exe,
            port: 0,
            idle_secs,
        })
    }
}

/// The server serving `root`, if any. Anything that answers on the recorded port, even with an
/// error, counts as running; only E_SERVER (no server.json, nothing listening) means none.
pub fn running(root: &Path) -> Option<ServerInfo> {
    let info = ServerInfo::read(root).ok()?;
    let answers = Ops::new(crate::client::Client::new(&info))
        .status()
        .map_or_else(|e| e.code != ErrorCode::Server, |_| true);
    answers.then_some(info)
}

/// Make sure a studio serves the project in `dir`, and open its tab once if asked.
pub fn ensure(dir: &Path, req: &EnsureRequest, launcher: &Launcher) -> CkResult<Ensured> {
    let root = dir.join(".clipkit");
    std::fs::create_dir_all(&root).map_err(|e| io_err(&root, e))?;
    // One ensure at a time per project, so concurrent hooks never start two servers.
    let lock_path = root.join("launch.lock");
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&lock_path)
        .map_err(|e| io_err(&lock_path, e))?;
    lock.lock().map_err(|e| io_err(&lock_path, e))?;

    let (info, started) = match running(&root) {
        Some(info) => (info, false),
        None => (start(dir, &root, launcher)?, true),
    };
    let opened = req.open && open_once(&root, &info)?;
    drop(lock);
    let ensured = Ensured {
        url: info.url,
        port: info.port,
        started,
        opened,
    };
    capped("studio", caps::STUDIO, ensured)
}

/// Spawn `ck studio` detached and wait until it answers.
fn start(dir: &Path, root: &Path, launcher: &Launcher) -> CkResult<ServerInfo> {
    let log_path = root.join("studio.log");
    let log = File::create(&log_path).map_err(|e| io_err(&log_path, e))?;
    let mut cmd = Command::new(&launcher.exe);
    cmd.arg("studio")
        .arg("--dir")
        .arg(dir)
        .arg("--port")
        .arg(launcher.port.to_string())
        .arg("--idle")
        .arg(launcher.idle_secs.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(log);
    detach(&mut cmd);
    let mut child = cmd
        .spawn()
        .map_err(|e| server_err(format!("cannot start {}: {e}", launcher.exe.display())))?;
    let t0 = Instant::now();
    loop {
        if child.try_wait().ok().flatten().is_some() {
            return Err(exited(&log_path));
        }
        if let Some(info) = running(root)
            && info.pid == child.id()
        {
            reap(child);
            tracing::info!(port = info.port, "studio started");
            return Ok(info);
        }
        if t0.elapsed() > READY_LIMIT {
            let _ = child.kill();
            let _ = child.wait();
            return Err(server_err(format!(
                "the studio did not answer within {} s",
                READY_LIMIT.as_secs()
            ))
            .with_hint("Run ck studio in a terminal to see why it does not start."));
        }
        std::thread::sleep(POLL);
    }
}

/// The new server runs on after this process exits, in its own process group so a Ctrl-C in
/// the agent's terminal does not stop it.
#[cfg(unix)]
fn detach(cmd: &mut Command) {
    use std::os::unix::process::CommandExt;
    cmd.process_group(0);
}

#[cfg(windows)]
fn detach(cmd: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    cmd.creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW);
}

#[cfg(not(any(unix, windows)))]
fn detach(_cmd: &mut Command) {}

/// Wait for a child in the background so a long-lived caller (`ck mcp`) leaves no zombie.
fn reap(mut child: Child) {
    std::thread::spawn(move || {
        let _ = child.wait();
    });
}

/// The server's own error (CkError JSON on its last stderr line), or the log tail.
fn exited(log_path: &Path) -> CkError {
    let text = std::fs::read_to_string(log_path).unwrap_or_default();
    let last = text
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("");
    if let Ok(e) = serde_json::from_str::<CkError>(last) {
        return e;
    }
    let tail: String = last.chars().take(LOG_TAIL).collect();
    server_err(format!("the studio exited at start: {tail}"))
        .with_hint("Read .clipkit/studio.log, fix the cause, then retry.")
}

/// Open the studio tab unless this server's tab was already opened. Returns whether it opened.
fn open_once(root: &Path, info: &ServerInfo) -> CkResult<bool> {
    let marker = root.join("opened");
    let id = format!("{}:{}", info.pid, info.port);
    if std::fs::read_to_string(&marker).is_ok_and(|m| m == id) {
        return Ok(false);
    }
    open_browser(&format!("{}#token={}", info.url, info.token))?;
    std::fs::write(&marker, id).map_err(|e| io_err(&marker, e))?;
    Ok(true)
}

/// `$BROWSER` (its first `:`-separated entry) or the platform opener, given the URL as one
/// argument. Never through a shell.
fn open_browser(url: &str) -> CkResult<()> {
    let custom = std::env::var("BROWSER")
        .ok()
        .and_then(|b| b.split(':').next().map(str::to_owned))
        .filter(|b| !b.trim().is_empty());
    let mut cmd = match custom {
        Some(program) => Command::new(program),
        None => platform_opener(),
    };
    let child = cmd
        .arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| {
            server_err(format!("cannot open a browser: {e}"))
                .with_hint("Open the url from ck studio --ensure with #token= from server.json.")
        })?;
    reap(child);
    Ok(())
}

#[cfg(target_os = "macos")]
fn platform_opener() -> Command {
    Command::new("open")
}

#[cfg(windows)]
fn platform_opener() -> Command {
    let mut cmd = Command::new("rundll32");
    cmd.arg("url.dll,FileProtocolHandler");
    cmd
}

#[cfg(not(any(target_os = "macos", windows)))]
fn platform_opener() -> Command {
    Command::new("xdg-open")
}

fn io_err(path: &Path, e: std::io::Error) -> CkError {
    server_err(format!("{}: {e}", path.display()))
}

fn server_err(message: String) -> CkError {
    CkError::new(ErrorCode::Server, message)
}

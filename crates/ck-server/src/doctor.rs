//! `doctor`: what this machine needs, each problem with one fix line. Key values are never read
//! out, only whether they are set.

use std::path::Path;
use std::process::{Command, Stdio};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod caps {
    pub const DOCTOR: usize = 2 * 1024;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DoctorCheck {
    pub name: String,
    pub ok: bool,
    /// Blocks work when false; optional checks only limit features.
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fix: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DoctorResult {
    pub ok: bool,
    pub checks: Vec<DoctorCheck>,
}

fn runs(program: &str, flag: &str) -> bool {
    Command::new(program)
        .arg(flag)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn check(name: &str, ok: bool, required: bool, fix: &str) -> DoctorCheck {
    DoctorCheck {
        name: name.into(),
        ok,
        required,
        fix: (!ok).then(|| fix.to_owned()),
    }
}

pub fn run(start: &Path) -> DoctorResult {
    let tool = |name: &str, flag: &str| {
        check(
            name,
            runs(name, flag),
            true,
            &format!("Install {name} and put it on PATH."),
        )
    };
    let checks = vec![
        tool("ffmpeg", "-version"),
        tool("ffprobe", "-version"),
        tool("node", "--version"),
        tool("git", "--version"),
        check(
            "render worker",
            crate::session::render::find_worker(start).is_ok(),
            true,
            "Reinstall ClipKit, or run just worker from a source checkout.",
        ),
        check(
            "FISH_AUDIO_API_KEY",
            std::env::var_os("FISH_AUDIO_API_KEY").is_some_and(|v| !v.is_empty()),
            false,
            "Optional: set it for AI voices and cloud transcription.",
        ),
    ];
    DoctorResult {
        ok: checks.iter().all(|c| c.ok || !c.required),
        checks,
    }
}

//! The ck binary. Phase 1: `--version`, a usage line and JSON logging setup.

use std::process::ExitCode;

const USAGE: &str = "usage: ck --version  (commands arrive in Phase 2 onward)";

/// JSON log lines to stderr when `CK_LOG` is set (for example `CK_LOG=debug`). Off by default so
/// command output stays small; fields follow ck_core::log.
fn init_logging() {
    let Ok(filter) = std::env::var("CK_LOG") else {
        return;
    };
    let _ = tracing_subscriber::fmt()
        .json()
        .flatten_event(true)
        .with_current_span(true)
        .with_span_list(false)
        .with_env_filter(tracing_subscriber::EnvFilter::new(filter))
        .with_writer(std::io::stderr)
        .try_init();
}

fn main() -> ExitCode {
    init_logging();
    tracing::debug!(version = env!("CARGO_PKG_VERSION"), "ck start");
    match std::env::args().nth(1).as_deref() {
        Some("--version" | "-V") => {
            println!("ck {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

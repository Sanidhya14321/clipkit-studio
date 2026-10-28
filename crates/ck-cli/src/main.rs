//! The ck binary. Phase 0 skeleton: only `--version` and a usage line.

use std::process::ExitCode;

const USAGE: &str = "usage: ck --version  (commands arrive in Phase 1 onward)";

fn main() -> ExitCode {
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

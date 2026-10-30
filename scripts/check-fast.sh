#!/usr/bin/env bash
# Fast check scoped to what changed since HEAD: format check, clippy and unit tests of changed crates.
# Prints a few lines; the full log goes to target/check-fast.log.
set -uo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
log="target/check-fast.log"
mkdir -p target
: >"$log"

changed="$( (git diff --name-only HEAD 2>/dev/null; git ls-files --others --exclude-standard) | sort -u)"

# Workspace-wide files invalidate every crate; otherwise only crates/<name>/ paths count.
if grep -qE '^(Cargo\.toml|Cargo\.lock|rust-toolchain\.toml|clippy\.toml|\.config/nextest\.toml)$' <<<"$changed"; then
  names="$(find crates -mindepth 2 -maxdepth 2 -name Cargo.toml | cut -d/ -f2 | sort)"
else
  names="$(grep -oE '^crates/[^/]+/' <<<"$changed" | cut -d/ -f2 | sort -u)"
fi
# Plain word splitting: macOS ships bash 3.2, which has no mapfile. Crate names have no spaces.
crates=($names)

fail() { echo "check-fast FAILED at $1 (full log: $log)"; grep -E '^(error|warning)|FAIL|panicked' "$log" | head -8; exit 1; }

if ! command -v cargo >/dev/null 2>&1; then
  echo "check-fast skipped: cargo not installed"
  exit 0
fi

cargo fmt --all --check >>"$log" 2>&1 || fail "cargo fmt --check (run: cargo fmt --all)"
if ((${#crates[@]} == 0)); then
  echo "check-fast ok (fmt only; no crate changes)"
  exit 0
fi

pkgs=()
for c in "${crates[@]}"; do [[ -f "crates/$c/Cargo.toml" ]] && pkgs+=(-p "$c"); done
((${#pkgs[@]})) || { echo "check-fast ok (fmt only)"; exit 0; }

cargo clippy "${pkgs[@]}" --all-targets --message-format short --quiet -- -D warnings >>"$log" 2>&1 \
  || fail "clippy"
if cargo nextest --version >/dev/null 2>&1; then
  cargo nextest run "${pkgs[@]}" --no-tests=pass --cargo-quiet >>"$log" 2>&1 || fail "tests"
else
  cargo test "${pkgs[@]}" --quiet >>"$log" 2>&1 || fail "tests"
fi
echo "check-fast ok (${crates[*]})"

#!/usr/bin/env bash
# Stop: run the fast check once. On failure, block the stop with a short summary so Claude fixes it.
# If stop_hook_active is true we are already continuing from this hook: never block again (no loops).
set -uo pipefail
input="$(cat)"
[[ "$(jq -r '.stop_hook_active // false' <<<"$input")" == "true" ]] && exit 0

root="${CLAUDE_PROJECT_DIR:-$(pwd)}"
# Nothing Rust-related changed: skip the check entirely so docs-only turns stay instant.
changes="$(cd "$root" && { git diff --name-only HEAD 2>/dev/null; git ls-files --others --exclude-standard; })"
grep -qE '(\.rs|Cargo\.toml|Cargo\.lock|rust-toolchain\.toml|clippy\.toml|rustfmt\.toml)$' <<<"$changes" || exit 0

if summary="$("$root/scripts/check-fast.sh" 2>&1)"; then
  exit 0
fi
jq -n --arg reason "$(head -c 1200 <<<"$summary")" '{decision: "block", reason: $reason}'
exit 0

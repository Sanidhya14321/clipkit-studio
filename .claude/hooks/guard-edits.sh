#!/usr/bin/env bash
# PreToolUse (Write|Edit|MultiEdit|NotebookEdit): block edits to protected paths.
set -uo pipefail
input="$(cat)"
file="$(jq -r '.tool_input.file_path // .tool_input.notebook_path // empty' <<<"$input")"
[[ -n "$file" ]] || exit 0
root="${CLAUDE_PROJECT_DIR:-$(jq -r '.cwd // empty' <<<"$input")}"
rel="${file#"$root"/}"
base="${file##*/}"

deny() { echo "Blocked: $1" >&2; exit 2; }
case "$rel" in
  docs/spec/*) deny "docs/spec/ is generated from docs/MASTER_PROMPT.md; never edit it." ;;
  docs/MASTER_PROMPT.md) deny "docs/MASTER_PROMPT.md is the frozen spec; record changes in DECISIONS.md instead." ;;
  assets/quarantine/*) deny "assets/quarantine/ holds untrusted third-party sources; they are read-only." ;;
esac
case "$base" in
  .env.example) ;;
  .env | .env.*) deny "$base may hold secrets; edit .env files by hand." ;;
esac
exit 0

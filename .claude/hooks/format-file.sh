#!/usr/bin/env bash
# PostToolUse (Write|Edit): format only the edited file. Silent on success; never blocks.
set -uo pipefail
file="$(jq -r '.tool_input.file_path // empty')"
[[ -n "$file" && -f "$file" ]] || exit 0
root="${CLAUDE_PROJECT_DIR:-$(pwd)}"
rel="${file#"$root"/}"

if [[ "$rel" == *.rs ]]; then
  rustfmt --edition 2024 --config-path "$root/rustfmt.toml" "$file" 2>/dev/null \
    || echo "rustfmt failed on $rel (syntax error?)" >&2
  exit 0
fi

# Studio and render worker: use the package's own prettier once it is installed (Phase 2+).
web_re='^(studio|render-worker)/.*\.(ts|tsx|js|jsx|mjs|css|json|html)$'
if [[ "$rel" =~ $web_re ]]; then
  prettier="$root/${BASH_REMATCH[1]}/node_modules/.bin/prettier"
  [[ -x "$prettier" ]] && "$prettier" --log-level warn --write "$file" >&2
fi
exit 0

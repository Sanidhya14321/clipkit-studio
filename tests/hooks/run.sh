#!/usr/bin/env bash
# Replay recorded hook payloads against .claude/hooks/*.sh and assert exit codes and speed.
set -uo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
hooks="$root/.claude/hooks"
export CLAUDE_PROJECT_DIR="$root"
pass=0
failures=0

# expect HOOK EXIT PAYLOAD [OUTPUT_REGEX]  (stdout+stderr must match OUTPUT_REGEX when given)
expect() {
  local hook="$1" want="$2" payload="$3" pattern="${4:-}" out code start ms
  start=$(date +%s%N)
  out="$("$hooks/$hook" <<<"$payload" 2>&1)"
  code=$?
  ms=$(( ($(date +%s%N) - start) / 1000000 ))
  if [[ "$code" != "$want" ]] || [[ -n "$pattern" && ! "$out" =~ $pattern ]] || ((ms > 2000)); then
    echo "FAIL $hook exit=$code want=$want ${ms}ms: $payload"
    [[ -n "$out" ]] && echo "     output: ${out:0:200}"
    failures=$((failures + 1))
  else
    pass=$((pass + 1))
  fi
}
edit() { printf '{"hook_event_name":"PreToolUse","tool_name":"Edit","cwd":"%s","tool_input":{"file_path":"%s"}}' "$root" "$1"; }
bash_cmd() { jq -cn --arg c "$1" --arg cwd "$root" '{hook_event_name:"PreToolUse",tool_name:"Bash",cwd:$cwd,tool_input:{command:$c}}'; }

# guard-edits: protected paths are blocked with a reason, everything else passes silently.
expect guard-edits.sh 2 "$(edit "$root/docs/spec/04-pipeline.md")" 'docs/spec'
expect guard-edits.sh 2 "$(edit "$root/docs/MASTER_PROMPT.md")" 'frozen spec'
expect guard-edits.sh 2 "$(edit "$root/.env")" 'secrets'
expect guard-edits.sh 2 "$(edit "$root/studio/.env.local")" 'secrets'
expect guard-edits.sh 2 "$(edit "$root/assets/quarantine/repo/README.md")" 'quarantine'
expect guard-edits.sh 0 "$(edit "$root/.env.example")" '^$'
expect guard-edits.sh 0 "$(edit "$root/docs/spec-notes.md")" '^$'
expect guard-edits.sh 0 "$(edit "$root/crates/ck-core/src/lib.rs")" '^$'
expect guard-edits.sh 0 "$(edit "$root/PROGRESS.md")" '^$'

# guard-bash: destructive deletes outside the repo and secret dumps are blocked.
for c in 'rm -rf /' 'rm -rf ~' 'rm -rf ~/Desktop' 'rm -rf ..' 'rm -rf ../other' 'sudo rm -rf /usr/lib' \
  'cd x && rm -fr "$HOME"' 'rm -r /etc/foo'; do
  expect guard-bash.sh 2 "$(bash_cmd "$c")" 'outside the repository'
done
for c in 'env' 'printenv' 'printenv FISH_API_KEY' 'echo $FISH_API_KEY' 'echo ${GITHUB_TOKEN}' 'cat .env' \
  'grep KEY studio/.env.local' 'cat /proc/self/environ' 'ls; set'; do
  expect guard-bash.sh 2 "$(bash_cmd "$c")" 'secrets'
done
for c in 'rm -rf target' "rm -rf $root/target" 'rm -rf /tmp/ck-test' 'cargo test' 'git status' \
  'cat .env.example' 'just check' 'rm file.txt' 'env RUST_LOG=debug cargo run' 'export FOO=1' \
  $'cat > .gitignore <<\'EOF\'\n.env\n.env.*\nEOF'; do
  expect guard-bash.sh 0 "$(bash_cmd "$c")" '^$'
done

# format-file: never blocks, silent on success, ignores files it does not own.
expect format-file.sh 0 "$(edit "$root/crates/ck-core/src/lib.rs")" '^$'
expect format-file.sh 0 "$(edit "$root/README.md")" '^$'
expect format-file.sh 0 '{"tool_input":{}}' '^$'

# stop-check: an active stop hook must never block again (loop guard), whatever the tree state.
expect stop-check.sh 0 '{"hook_event_name":"Stop","stop_hook_active":true}' '^$'

echo "hooks: $pass passed, $failures failed"
((failures == 0))

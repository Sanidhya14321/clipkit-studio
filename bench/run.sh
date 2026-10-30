#!/usr/bin/env bash
# One benchmark run (spec 10.3): a fresh workspace per arm, scenario and run, the agent driven
# by AGENT_CMD (for example: claude -p --output-format json), then `ck eval` on the export.
# Appends one JSON line to bench/results.jsonl. Not run in CI: it needs a real agent and tokens.
#
#   AGENT_CMD='claude -p --output-format json' bench/run.sh B generated-teaser 1
set -euo pipefail
arm="$1"; scenario="$2"; run="$3"
: "${AGENT_CMD:?set AGENT_CMD to the agent command; the prompt is passed as the last argument}"
root="$(cd "$(dirname "$0")/.." && pwd)"
ws="$(mktemp -d)"
prompt="$(jq -r --arg s "$scenario" '.scenarios[] | select(.id == $s) | .prompt' "$root/bench/scenarios.json")"
expect="$(jq -c --arg s "$scenario" '.scenarios[] | select(.id == $s) | .expect // {}' "$root/bench/scenarios.json")"
if [ "$arm" != "A0" ] && [ "$arm" != "A1" ]; then (cd "$ws" && ck init --yes >/dev/null); fi
start=$(date +%s)
out="$(cd "$ws" && $AGENT_CMD "$prompt" || true)"
wall=$(( $(date +%s) - start ))
# Tokens only when the agent reports them (Claude Code JSON output: usage); else not measured.
tokens="$(printf '%s' "$out" | jq -r '[.usage.input_tokens, .usage.output_tokens] | add // empty' 2>/dev/null || true)"
video="$(ls "$ws"/exports/*.mp4 2>/dev/null | head -1 || true)"
if [ -n "$video" ]; then
  printf '%s' "$expect" > "$ws/expect.json"
  score="$(cd "$ws" && ck eval "$video" --expect expect.json | jq '.score')"
else
  score=0
fi
jq -nc --arg arm "$arm" --arg scenario "$scenario" --argjson run "$run" \
  --argjson wall "$wall" --argjson score "$score" --arg tokens "$tokens" \
  '{arm: $arm, scenario: $scenario, run: $run, wallSec: $wall, evalScore: $score,
    tokens: (if $tokens == "" then null else ($tokens | tonumber) end)}' \
  >> "$root/bench/results.jsonl"

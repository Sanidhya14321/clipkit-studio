#!/usr/bin/env bash
set -uo pipefail
MAX_RUNS=${MAX_RUNS:-60}          # hard cap on sessions
MAX_TURNS=${MAX_TURNS:-80}        # cap per session (verify flag)
MAX_NO_PROGRESS=${MAX_NO_PROGRESS:-2}
PAUSE_AFTER_PHASE=${PAUSE_AFTER_PHASE:-0}   # 1 = stop after each phase for review
mkdir -p .autobuild
no_progress=0
for i in $(seq 1 "$MAX_RUNS"); do
  [ -f .autobuild/STOP ] && { echo "STOP file found"; break; }
  status=$(grep -m1 '^STATUS:' PROGRESS.md | awk '{print $2}')
  case "$status" in DONE|BLOCKED|PAUSED) echo "Status $status"; break;; esac
  phase_before=$(grep -m1 '^PHASE:' PROGRESS.md)
  head_before=$(git rev-parse HEAD)

  claude -p "$(cat scripts/autobuild-prompt.md)" \
    --output-format json --max-turns "$MAX_TURNS" \
    --permission-mode acceptEdits \
    > ".autobuild/run-$i.json" 2> ".autobuild/run-$i.err"

  check_ok=1
  just check > ".autobuild/check-$i.log" 2>&1 || { check_ok=0; echo "check failed after run $i" >> .autobuild/alerts.log; }
  if [ "$(git rev-parse HEAD)" = "$head_before" ]; then
    no_progress=$((no_progress+1))
  else
    no_progress=0
    # Publish green progress to GitHub. Plain push only, never forced; the agent itself never pushes.
    if [ "${PUSH:-1}" = 1 ] && [ "$check_ok" = 1 ]; then
      git push origin HEAD:main > ".autobuild/push-$i.log" 2>&1 || echo "push failed after run $i" >> .autobuild/alerts.log
    fi
  fi
  [ "$no_progress" -ge "$MAX_NO_PROGRESS" ] && { echo "No commits in $no_progress runs"; break; }
  if [ "$PAUSE_AFTER_PHASE" = 1 ] && [ "$(grep -m1 '^PHASE:' PROGRESS.md)" != "$phase_before" ]; then
    echo "Phase finished; pausing for review"; break
  fi
done

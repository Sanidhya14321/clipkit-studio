---
name: debug-triage
description: Use when something breaks - a failing test, an error code, a crash or wrong output. Read the error, run ck doctor, open the trace, bisect, then fix with a failing test first.
---

# Debug triage

Never fix blindly. Find the cause first, prove it with a test, then fix.

1. Read the error code and hint (`E_*`). Look it up in the playbook (spec 10.4) for the first
   command to run.
2. Environment: `ck doctor` (ffmpeg, ffprobe, Node, ports, disk, keys, adapters). Before ck
   exists, check tool versions by hand.
3. Logs: structured JSON logs carry `trace_id, run_id, turn_id, stage, pass, layer_id, rev`.
   Filter with `rg` or `jq` by the run id; never read whole logs into context.
4. Trace: open `.clipkit/trace/<RUN_ID>/` (events, patches, snapshots, ffmpeg logs).
5. Narrow: `ck repro` (smallest failing project), `ck replay` (rebuild from patches without an
   agent), `ck bisect` (first revision where a check fails). Use `git bisect` for code regressions.
6. Write a test that fails for the cause. Fix. Watch it pass. Run `just check`.
7. Record the symptom, cause and fix in DECISIONS.md (one short entry).

Rules:
- Two failed attempts at the same approach -> stop and ask the researcher subagent or the human.
- For noisy suites use the test-runner subagent.
- Never weaken or delete a test to make it pass; quarantine flaky tests with their trace.

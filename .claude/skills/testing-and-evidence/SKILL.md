---
name: testing-and-evidence
description: Use when deciding which test layer to write, using fixtures or cassettes, or recording evidence in VERIFICATION.md.
---

# Testing and evidence

## Pick the layer (spec 10.1)
- Pure logic (patches, timeline math, locks, stage machine, guardrails): Rust unit tests;
  proptest for apply/undo/redo/conflicts and timeline math; insta for schemas and outputs.
- Every CLI command and MCP tool: contract test for input/output schema, an output size cap,
  and CLI/MCP parity.
- Server + WebSocket, ffmpeg on fixture media, ingestion, export: integration tests in the crate's
  `tests/` folder.
- Studio behaviour: Playwright (`just e2e`). Hooks: `tests/hooks/run.sh` with recorded payloads.
- Real agents in the loop and the A0/A1/B benchmark: on demand only (`just bench`), costs tokens.

## Fixtures and cassettes
- `just fixtures` generates synthetic media plus `ground-truth.json`; tests read expectations from it.
- Provider calls (TTS, STT, cloning, GitHub) use recorded cassettes with record, replay and live
  modes. Default is replay; tests never need network or keys.
- Owner-supplied real media is PLACEHOLDER until provided; tests needing it are skipped with a
  clear reason, never faked.

## Running
- `just check` (fmt, clippy, fast tests) after each step. `just test` for the full suite.
- Use the test-runner subagent for long runs; it returns only failures.
- Flaky: anything under 5/5 passes is quarantined with its trace, never deleted.

## Evidence (VERIFICATION.md)
- Each item: status PASS, FAIL or BLOCKED, plus evidence: a test name, a command with its
  output path, or a screenshot path. No PASS without evidence. Never report a test you did not run.

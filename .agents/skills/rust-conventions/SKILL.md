---
name: rust-conventions
description: Use when writing or reviewing Rust in crates/. Crate boundaries, stable error codes, logging fields, serde and schemars patterns, async rules, and adding a CLI command with its MCP twin.
---

# Rust conventions

## Crate boundaries (dependencies point downward only)
- `ck-core`: model, patches, revisions, locks, migrations. No IO beyond atomic file storage, no async.
- `ck-pipeline` -> core. `ck-media`, `ck-assets`, `ck-voice` -> core (+ pipeline types if needed).
- `ck-server`, `ck-mcp` -> the engines. `ck-cli` -> everything; it is the only binary.
- `ck-eval` reads exported files only; it must not depend on server or pipeline internals.
- Shared operations live in a library function; CLI and MCP are thin adapters over it.

## Errors
- One `thiserror` enum per crate; every variant maps to a stable code (`E_STAGE`, `E_LOCK_HELD`,
  `E_PATCH_STALE`, `E_SCOPE`, `E_BUDGET`, ...) and a one-line hint. Codes never change once shipped.
- `anyhow` only in `ck-cli` main and tests. No `unwrap`/`expect` in library code (clippy denies
  `unwrap_used`); tests may unwrap.

## Types
- Every public type: `#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]` plus
  `#[serde(rename_all = "camelCase")]`; TS bindings are generated (never hand-written).
- Times in seconds (`f64`); convert to frames only at render time. Ids are short stable strings.

## Logging
- `tracing` with JSON output. Every span/event carries what applies of:
  `trace_id, run_id, turn_id, stage, pass, layer_id, rev`. Never log secrets or media contents.

## Async
- tokio only in server, mcp, cli, voice, media. Core and pipeline stay sync and pure.
- No blocking IO on async tasks: use `spawn_blocking` or `tokio::process` for ffmpeg.

## Adding a command (always both twins)
1. Library fn in the owning crate with input/output types (serde + JsonSchema).
2. `ck-cli` subcommand (clap) and `ck-mcp` tool calling the same fn.
3. Tests: schema snapshot (insta), output size cap, CLI/MCP parity on the same input.

## Style
- Small files, one concept each. `cargo fmt` runs automatically on edit.
- Prefer `--message-format short` and `cargo nextest` for terse output.

# Decisions

Newest first within each section. Each entry: what, why, and the source when it is a verified fact.

## Phase 4 (2026-10-29)

### Step 2: taste and look commands (spec 6.2)
- **`ck taste read`, not `get`**: spec 6.2 names `taste search, read`; the Phase 4 plan said get. Looks have no spec 6.2 entry; `ck look list|get` were added because the brief's look cards (spec 8.1) and re-theming need them.
- **Built into the binary**: taste files and looks are embedded with `include_str!` so an installed `ck` needs no repo `assets/` folder; a test checks the embedded copy equals the folder. Adding a file means adding it to the table in `ck-assets/src/library.rs`.
- **Search is word scoring, not tantivy**: 12 files do not need an index; 3 points per query word found in id, title or tags, 1 in the body, ties in id order. Default 5 hits, at most 12. Tantivy arrives with cards (Phase 8).
- **No studio needed**: these four commands and their MCP twins read the built-in library and never start the studio. Unknown ids are `E_NOT_FOUND` (its generic hint names `ck project get`; the message names the right list command).

### Step 1: taste library and looks (spec 7.1, 7.5)
- **Twelve taste files**: the ten topics spec 7.1 names (pacing, hooks, typography, color, motion, transitions, captions, platform specs, audio mixing, recipes for common video types as `video-types`) plus `layout` and `sound-design`, which spec 8.5 (layout solver, sound placement) needs numbers for. All numbers are ClipKit's own authored defaults, not platform-official figures; `platform-specs` says so.
- **Taste format**: front matter with only `id`, `title`, `tags: [..]` parsed by hand (no YAML crate), id equals file name, at most 2400 bytes (about 600 tokens) and the body must contain numbers.
- **Look format**: JSON (serde, `deny_unknown_fields`) with six color roles, two font roles (so at most two families, spec 4.3), type scale ratio, radius and spacing as fractions of frame height, and motion (personality, duration scale, enter/exit easing, stagger). Validation: every text role on its background reaches WCAG 4.5 to 1.
- **Fonts** are named Google Fonts families (Inter, Archivo Black, Playfair Display, Source Serif 4, Source Sans 3); no font files are bundled yet. Licenses and files are checked when fonts are ingested (Phase 8) or loaded by the render path (step 7).
- **New error code `E_ASSET`** (HTTP 500) for malformed knowledge base files; file read failures stay `E_STORAGE`.

## Phase 3 (2026-10-29)

### Step 7: Cursor and any MCP client (spec 6.3)
- **Format UNVERIFIED**: the researcher could not reach cursor.com (web tools denied in headless runs). From memory: global MCP servers in `~/.cursor/mcp.json` as `{"mcpServers":{"clipkit":{"command":"ck","args":["mcp"]}}}`; rules as `.cursor/rules/*.mdc` with `description`/`globs`/`alwaysApply` frontmatter; user-level rules live only in Cursor's settings, not in a file. Check this against cursor.com/docs/context/mcp and /rules in an interactive session.
- **Rules go in the project folder**: since there is no user-level rules file, `ck init --agent cursor` writes `<--dir>/.cursor/rules/clipkit.mdc` (default: current folder). Uninstall removes it only from the folder it runs in, and only while it still holds ClipKit's text; a file the user edited stays. The MCP entry is user-level and is merged like Claude Code settings: a `clipkit` server of the user's own is refused by init and kept by uninstall.
- **MCP server folder**: the Cursor and generic entries run plain `ck mcp` (like Codex), so the project is the MCP server's working folder. Cursor's `${workspaceFolder}` interpolation is not used until it is verified.
- **Any MCP client** (`--agent mcp`): clients have no common config path, so ck writes `~/.clipkit/mcp/mcp.json` (common `mcpServers` shape) and `~/.clipkit/mcp/RULES.md` (the Codex section text) for the user to point their client at. It is never detected; it is written only when asked for, and removed by uninstall.
- **Rules text** is the one in `adapters/codex/AGENTS.md`, shared by Codex, Cursor (after `adapters/cursor/frontmatter.mdc`) and the generic RULES.md.
- Uninstall also removes an agent folder (`~/.cursor`, `~/.claude`) that was empty before init: that is the existing rule of removing a folder when its last file is removed. Kept as is.

### Step 6: proof and review
- **Headless proof stand-in**: a real Claude Code or Codex session cannot run headless, so `studio/e2e/phase3.spec.ts` drives the same pieces Claude Code would: the hooks.json command lines through `sh -c` with `CLAUDE_PROJECT_DIR`, and `ck mcp --dir <project>` as .mcp.json registers it. The real-session proof stays BLOCKED in VERIFICATION.md, not marked PASS.
- **Hook vs start time**: the prompt hook's timeout stays 10 s. A start takes about 16 ms; the 10 s ready limit is only reached by a studio that hangs, and then Claude Code kills the hook, which is non-blocking. The hook no longer retries a failed start (that could double the wait); it falls back only to a studio that is running (browser failure).
- **Launch lock**: waits at most 12 s (ready limit plus margin), then E_SERVER with a hint. Was unbounded.
- **Kept as is (reviewer, low)**: the token reaches the browser on the opener's argv (`url#token=`, spec 5.1 opens the tab with it; local `ps` can read it, same user model as server.json); `$BROWSER` is split on `:` (Unix convention; Windows uses the platform opener); the `pid:port` marker could repeat after pid reuse (the tab then does not reopen; a token-derived marker can replace it later); `studio.log` is truncated per start (only under the launch lock, so a running server's log is not lost in practice); guard misses `sh -c`, `python -c`, `eval` (it is a heuristic; the server enforces guardrails).
- **Spec 6.3 Cursor and generic MCP clients**: Phase 3 reads spec 06 and no later phase owns it, so it becomes Phase 3 step 7 instead of staying deferred (replaces the step 5 note).

### Step 5: `ck init` / `ck uninstall`
- **Shape**: `ck init|uninstall [--agent claude-code|codex]... [--yes]`. Init's default is the detected agents (config folder exists or the command is on PATH); uninstall's default is every agent, since it only looks for what init added. Output: one JSON line `{command, applied, agents: [{agent, steps: [{path, change}]}], hint?}`, cap 2048 bytes.
- **Asking**: without `--yes`, the plan goes to stderr and a y/N question is asked when stdin is a terminal; with no terminal nothing is written and the plan is the output (`applied: false`, hint names `--yes`). Agents run ck init with `--yes` only after the user agreed in chat.
- **No MCP twin**: init and uninstall change agent configuration on the user's machine; like `ck hook`, they are install-time adapters, not agent operations (spec 6.2 does not list them).
- **Claude Code** (format verified 2026-10-29 on code.claude.com/docs/en/plugins/marketplace-reference, create-marketplace, settings): the binary embeds `plugin/` (README excluded) and writes it as a local marketplace at `~/.clipkit/claude-plugin` (`.claude-plugin/marketplace.json`, plugin in `./clipkit`), plus the plugin's `.mcp.json` from a constant (the repo file is still blocked, step 4). `settings.json` (in `$CLAUDE_CONFIG_DIR` or `~/.claude`) gets `extraKnownMarketplaces.clipkit = {source: {source: "directory", path}}` and `enabledPlugins["clipkit@clipkit"] = true`. The docs do not say whether hand-written keys install the plugin without `claude plugin install`: UNVERIFIED, checked in the step 6 proof run. A `clipkit` marketplace that points elsewhere is the user's: init refuses, uninstall leaves it.
- **settings.json rewrite**: serde_json without `preserve_order` (turning it on reordered the ck-core schema snapshot, since features unify across the workspace), so a rewritten file is pretty-printed with sorted keys. Values are kept; the file is rewritten only when a ClipKit key changes. Unparsable settings (for example JSONC) are refused by init and skipped by uninstall.
- **Codex** (UNVERIFIED: the OpenAI docs could not be fetched in this run): `$CODEX_HOME` or `~/.codex`; `config.toml` gets `[mcp_servers.clipkit] command = "ck", args = ["mcp"]` and `AGENTS.md` gets a short section (adapters/codex/), each between marker lines. No `--dir`: Codex is assumed to start MCP servers in the session folder. A `mcp_servers.clipkit` the user defined in any spelling (line scan) is refused. The AGENTS.md section is short because a global AGENTS.md is read in every Codex session (spec 9).
- **No residue**: blocks are cut out exactly; a file left empty is removed, and so is a folder whose last file uninstall removed. Known limit: when a user file did not end in a newline, init adds one before its block and uninstall keeps it. Writes go through symlinks and keep the file mode.
- **Not atomic across files**: an I/O error midway leaves part of the plan applied; running the command again is safe because the plan is recomputed from disk (the error hint says so).
- **Cursor and generic MCP**: not written (Phase 3 names Claude Code and Codex; spec 6.3's Cursor line has no phase yet).

### Step 4: `plugin/`
- **Format** checked on 2026-10-29 against code.claude.com/docs/en/plugins-reference, /hooks, /mcp, /sub-agents and /skills (researcher). The hook input field names chosen in step 3 match the reference, so no change to `ck hook`. SessionEnd hooks get about 1.5 s by default; `session-end` makes one loopback call, so that is enough.
- **`ck` is found on PATH** (the release binary or npm wrapper installs it, spec 2.4); the plugin ships no binary and does not use `${CLAUDE_PLUGIN_ROOT}`. A missing `ck` makes a hook fail with a non-2 exit, which Claude Code treats as non-blocking, so a session never stops for it.
- **Project folder**: hooks pass `--dir "$CLAUDE_PROJECT_DIR"`. MCP passes `--dir ${CLAUDE_PROJECT_DIR}` too, because the reference does not say which working directory a plugin MCP server starts in.
- **Timeouts** (seconds): prompt 10 (an ensure is about 16 ms, but a cold browser opener can be slow), guard 5, stop 5, session-end 2.
- **Matcher** `Write|Edit|Bash` is the spec's list. `ck hook guard` also understands MultiEdit and NotebookEdit, but the matcher keeps to the spec's tools; widen it if Claude Code still sends those tools.
- **Manifest**: name `clipkit`, version 0.1.0, license `PLACEHOLDER` (the project license is not decided for the plugin); no author field (the reference makes it optional).
- **Skill text**: clipkit-video carries the spec 9.4 text unchanged, plus frontmatter and one REFERENCES line that links pipeline.md, guardrails.md and voice.md and notes the MCP naming. A test reads the 9.4 block from the spec, so the skill cannot drift from it. The reference files summarize spec 4 and spec 6.4/8.3, and guardrails.md names the real error codes. Commands that do not exist yet (pipeline, brief, verify and others) are named as the spec names them; they arrive in Phase 4 and later.
- **Critic**: `tools: Read` only (it reads the contact sheet and the rubric), `model: inherit`.
- **`.mcp.json` BLOCKED**: the headless run's permission layer refused to write `plugin/.mcp.json` as a sensitive file. That refusal was not worked around (no shell write, no inline `mcpServers` in plugin.json). Intended content: `{"mcpServers": {"clipkit": {"command": "ck", "args": ["mcp", "--dir", "${CLAUDE_PROJECT_DIR}"]}}}`. Its test is `#[ignore]`d until an interactive session writes the file and removes the ignore.

### Step 3: `ck hook prompt|guard|stop|session-end`
- **Protocol**: each hook reads Claude Code's hook JSON from stdin (1 MiB cap; malformed or oversize input counts as empty) and uses only `prompt`, `cwd`, `tool_name`, `tool_input`, `stop_hook_active`. Exit 0 lets the agent go on; exit 2 blocks with one line on stderr. A hook never exits 1, so an internal error never disrupts a session (logged under CK_LOG). Field names are from the Claude Code hooks reference as known; step 4 checks them against the current reference.
- **Project folder is `--dir`** (hooks.json in step 4 will pass `$CLAUDE_PROJECT_DIR`); the input's `cwd` only resolves relative file paths in guard, since the agent may `cd` into a subfolder.
- **Video prompt = word list** (video, footage, reel, trailer, montage, voiceover, b-roll, YouTube, mp4 and similar; spec 5.1 "looks like a video request"). Ambiguous coding words (clip, edit, render, timeline, captions) are left out so ordinary prompts stay untouched. An existing `.clipkit/project.json` also triggers it. The hook ensures with open; a failing browser falls back to ensure without open. The note is one line of at most 300 bytes: url, mode, stage, agent status. If the start fails it prints one line with the error code instead.
- **Guard is active only when `.clipkit/` exists.** File tools (Write, Edit, MultiEdit, NotebookEdit) are refused on any `.clipkit` path component, `<dir>/assets/` and media extensions (video, audio, raster images; SVG is text and allowed), with paths normalized lexically. Bash is read word by word per segment (split on `; | & ( ) { }` and backticks): any `.clipkit` word, ffmpeg/ffprobe/ffplay/sox as the program (after env assignments and wrappers like sudo or timeout), and mutating commands (rm, mv, tee, `sed -i`, the destination of cp, redirects) aimed at assets/ or media files. Reads of assets/ and mentions like `which ffmpeg` pass. It is a heuristic; the server enforces the guardrails.
- **Stop "stage incomplete" = agent status `working`** (no pipeline commands exist until Phase 4; this is the state they will set mid-stage), plus any live agent lock. It never starts a studio (probe only), and `stop_hook_active` makes it step aside, so it blocks at most once per stop.
- **SessionEnd** releases every agent lock if a studio answers. "Flush the trace": there is no trace store yet (logs go to stderr under CK_LOG); the trace bundle arrives with `ck trace bundle` (Phase 9).
- **No MCP twin**: hooks are Claude Code's event adapters, not agent operations; an MCP client has no hook events to forward.

### Step 2: the first ck or MCP call ensures the studio
- **`launch::connect(dir, launcher)`** serves every ops command (`status`, `project get|patch`, `lock`, `unlock`, `heartbeat`) and every MCP tool except `studio`. It probes the recorded server (one extra loopback status call, about 1 ms) and, if none answers, runs the same locked ensure as `ck studio --ensure` with the default idle time and a free port. It probes instead of retrying an op after E_SERVER, so a patch is never sent twice.
- **The tab opens on an auto-start, best effort**: spec 5.1 says hookless agents "get the same behavior", and the hook passes `--open`. The tab opens only when this call started the server (once per server, the same marker), and a failing or missing browser is logged, not returned: the agent's command must not fail because no browser exists (CI, SSH). A server the user started in the foreground gets no extra tab.
- **E_SERVER now means the start failed** (or a server died mid-call); the server's own start error is relayed, as for `--ensure`. Commands in a folder with no project create `.clipkit` and an empty project, like `ck studio`. No opt-out flag (the spec asks for none).
- Tests set `BROWSER=true` in the shared ck helpers so test runs never open a real browser.

### Step 1: `ck studio --ensure`
- **Ensure lives in ck-server (`launch`)**, so the CLI and the MCP `studio` tool share it. It spawns this process's own binary (`current_exe`) as `ck studio` in a new process group (Windows: new process group, no window), stdout null, stderr to `.clipkit/studio.log`. It returns once the new pid answers status; measured 16 ms cold, 5 ms warm (spec 5.1 asks about 200 ms). Waiting for the answer instead of returning right after spawn keeps the next ck call from racing the server start.
- **Concurrent ensures** are serialized with an OS file lock on `.clipkit/launch.lock` (std `File::lock`, Rust 1.89+), so parallel hooks start one server. This narrows, but does not close, the Phase 2 check-then-bind gap for a foreground `ck studio` racing an ensure.
- **"Once per session" = once per server**: `.clipkit/opened` records `pid:port` of the server whose tab was opened; a restarted server opens a tab again. No WebSocket-presence check (status has no tab count; adding one would change the generated types for little gain).
- **Opener**: `$BROWSER` (first `:`-separated entry, run as a program with the URL as one argument, never a shell), else `xdg-open` / `open` / `rundll32 url.dll,FileProtocolHandler`. `$BROWSER` is the usual convention and the test seam (a fake script records the URL). The token travels only in the opener's argument, never on stdout.
- **MCP twin**: tool `studio` (input `{open}`) always ensures, since an MCP call cannot serve in the foreground. `--quiet` has no MCP equivalent (a tool must return a result). Output cap `caps::STUDIO` = 1024 bytes.
- Without `--quiet` ensure prints `{url, port, started, opened}`; `--open` and `--quiet` require `--ensure` (usage error otherwise).

## Asset supply expansion (2026-10-28, owner update)

Source: docs/sources/asset-expansion-catalog.md, treated as third-party data. Spec: docs/spec/14-asset-expansion.md.

### Routes for the catalog's Python tools (single Rust binary, no Python runtime)
Crates checked on crates.io on 2026-10-28. Each is re-verified when its phase starts; nothing is added to Cargo.toml yet.
- **Transcription (faster-whisper)**: `whisper-rs` 0.16.0 (2026-03-12, Unlicense; repo moved to codeberg.org/tazz4843/whisper-rs) binding whisper.cpp (MIT). Builds whisper.cpp from C++ source, so the build needs a C++ toolchain and cmake. The binary stays single; the ggml model file is downloaded on first use, pinned per project and cached. Local whisper is the default, Fish Audio STT the cloud option. Model weight license to be read at Phase 5.
- **VAD (Silero)**: `ort` has no stable 2.0 release; candidates are 2.0.0-rc.14 (2026-10-12) or rc.13 (2026-07-28); rc.15 (2026-10-19) is too new. MIT OR Apache-2.0, MSRV 1.92. Open point for Phase 5: ort downloads ONNX Runtime binaries at build time; confirm it links statically into `ck`, or evaluate a pure-Rust ONNX runtime (for example tract) instead. `voice_activity_detector` 0.2.1 was rejected: non-standard license and pinned to ort rc.10.
- **Scene detection (PySceneDetect)**: ffmpeg scene score (`scdet` filter or `select='gt(scene,T)'` with `showinfo`) from ck-media. No new crate.
- **Loudness (pyloudnorm)**: ffmpeg `ebur128` to measure and two-pass `loudnorm` to normalize, for every gate and export preset, so measurement and normalization share one meter. The `ebur128` crate (0.1.10, 2024-10-26, MIT; 0.1.11 from 2026-10-26 is too new) is kept as an optional in-process cross-check in tests only.
- **Onset and beat (librosa)**: deferred. `aubio` and `aubio-rs` were last released in 2021; `beat-detector` 0.1.2 is tiny and unmaintained. Revisit in Phase 9; until then pacing uses taste rules.
- **Stems (Demucs)**: deferred to Phase 10 as an optional opt-in sidecar, off by default, never in the interactive path, stems cached by source hash. The sidecar's runtime is an open question, because it must not add Python to the default install.
- JavaScript tools stay on the TypeScript side: tesseract.js, remotion-qa and audio-bed-check in the evaluator and CI (Phase 9, after license checks); wavesurfer.js, culori and opentype.js in the studio (Phase 9).

### sources.yaml
- Restored unchanged from `codex-archive` (38 entries; JSON is valid YAML), then extended. Existing entries are untouched except the `remotion` line, which gains a `packages` list.
- 14 `@remotion/*` packages confirmed on npm (4.0.539) were added to the `remotion` line. `@remotion/claude-code-plugin` and `@remotion/codex-plugin` were named by the catalog but return 404 on npm, so they were not added.
- 25 new entries: `status: unverified`, `enabled: false`, `licenseClass: unknown`, plus the claimed license in `license`, `wave`, `phase`, `homepage` for site sources and `licenseNote` for custom, site, CC BY, trademark and paid-tier notes.
- Repos left null where the catalog gives none or hedges: remotionui, remotion-bits, cliphouse, remotion-qa, audio-bed-check, origin-ui (two candidate orgs), kenney-audio, iwltbap-luts, pixabay-media.
- Kind and tier come from the catalog's prose because its YAML companion was not supplied.
- Pixabay is in wave 1, but ingesting it needs the Phase 8 vetting pipeline, so its phase is 8.

### Spec folder
- `docs/spec/14-asset-expansion.md` is an owner-requested addition. The guard-edits hook blocks Write and Edit under `docs/spec/` by design, so it was written through the shell at the owner's explicit request. `split-spec.py` only rewrites files 00 to 13.

## Phase 2 (2026-10-28)

### Crate versions (latest on crates.io per `cargo search` on 2026-10-28)
Release dates are PLACEHOLDER: this headless run could not reach the web (WebFetch and the researcher's tools were denied), so the two-week-age rule is unverified. Open question: re-check dates and pin back if any is too new.
| Crate | Version | Notes |
| --- | --- | --- |
| tokio | 1.54.1 | rt-multi-thread, macros, net, sync, time, signal |
| axum | 0.8.9 | default features off; http1, json, query, tokio, ws |
| getrandom | 0.4.3 | session token from the OS generator (`getrandom::fill`) |
| tower | 0.5.3 | dev, `util` for `oneshot` |
| tokio-tungstenite | 0.29.0 | dev; same major as axum's ws dependency (0.30.0 is newest) |
| futures-util | 0.3.34 | dev |

### Server choices (steps 1 and 2)
- **New error codes**: E_AUTH (token, Host or Origin refused), E_NOT_FOUND, E_REQUEST (malformed or oversized body), E_SERVER (bind, task or state failures). Added to the pinned list; existing codes unchanged.
- **Guards**: every route checks Host is `127.0.0.1:<port>` or `localhost:<port>` (DNS rebinding) and Origin, when present, is this server (cross-site). `/api/*` and `/ws` need the token: header `x-ck-token`; the `token` query parameter counts only on `/ws` (browsers cannot set WebSocket headers), so API tokens stay out of URLs.
- **server.json** holds port, token, pid and URL; written via a 0600 temp file and rename, removed on shutdown only if it still describes this server.
- **Events**: `snapshot` (whole project, on connect and after each change, only built when a subscriber exists), `revision` (rev, author, label, touched) and `locks`. The studio renders from snapshots; diffs over the wire can come later if size demands it.
- **Writes** run on `spawn_blocking` (they fsync). A failed save reloads the document from disk; if the reload also fails, the session refuses all calls with E_STORAGE.
- **Stage gate not wired yet**: agent patches are allowed only in BASE, PASSES and ITERATE, but no pipeline commands exist before Phase 4, so wiring it now would make every Phase 2 agent patch fail. It is wired together with the pipeline commands.
- **Trust boundary (open)**: callers state `author`/`holder` themselves; any token holder can patch or unlock as either role. Acceptable for one local user; per-role credentials (agent token vs studio token) are an open question for Phase 3, when the plugin and studio get distinct launch paths.
- Deferred (steps 1-2): idle shutdown, refusing a second server while server.json's pid is alive (`ck studio --ensure`, step 3 / Phase 3); redacting paths in 5xx bodies (local, same-user only).

### Step 3: shared ops and CLI basics
Crates (latest per `cargo search`/`cargo info` on 2026-10-28; release dates PLACEHOLDER, same headless limit as above):
| Crate | Version | Notes |
| --- | --- | --- |
| clap | 4.6.7 | `derive`; ck-cli only |
| ureq | 3.5.0 | default features off (no TLS, no gzip); blocking loopback client in ck-server |
- **Client**: ureq, blocking, because the CLI is sync and the server already owns the async runtime; the MCP twin calls it from `spawn_blocking`. Proxy from the environment off, redirects off (the token header never leaves), 30 s timeout, replies over 64 KB refused as E_OUTPUT_CAP.
- **Shared ops live in `ck-server::ops`** (thin typed calls through `client`); the CLI and MCP adapt them. The CLI talks to the running studio over HTTP so every write passes the same session gates.
- **Output caps** (compact JSON bytes): status 4 KB, project summary 16 KB, layer 8 KB, patch 4 KB, lock/unlock 4 KB. Reads over the cap fail with the new code E_OUTPUT_CAP. Writes are already committed when the reply arrives, so their list (`touched` or `locks`) is trimmed to the prefix that fits and `omitted` counts the rest (refusing would make a retry fail as stale). Help text gets 2 KB.
- **Command names**: spec 6.2 groups `project get --summary, get --layer, patch; lock, unlock`, read as `ck project get`, `ck project patch`, `ck lock`, `ck unlock`. `ck project get` with no flag returns the summary. `--holder` defaults to agent. The patch argument is the full PatchRequest JSON, or `-` for stdin (1 MB limit).
- **`ck studio`** serves in the foreground on 127.0.0.1 (`--port`, default free port), prints `{url, port}` (never the token) and stops on Ctrl-C or SIGTERM. With no project it creates an empty one (id `project`, named after the folder, mode GENERATE until mode detection in Phase 4). It refuses to start when anything answers on the port in server.json with other than E_SERVER. `--ensure`, `--open`, `--quiet` and background start are Phase 3 (auto-launch); until then the E_SERVER hint names a flag that does not exist yet.
- **CLI output**: one line of compact JSON on stdout; errors are the CkError JSON on stderr with exit 1; clap usage errors exit 2.

### Step 4: MCP twin (2026-10-29)
| Crate | Version | Notes |
| --- | --- | --- |
| rmcp | 3.8.0 | latest per `cargo info` on 2026-10-29; release date PLACEHOLDER (web lookups denied in the headless run, researcher subagent included); default features off, `server` + `transport-io` only (no HTTP, no auth) |
- **Handler without macros**: `ServerHandler` is implemented by hand over a plain tool table in `ck-mcp::tools` (name, CLI twin, description, cap, input schema from schemars), so the table is testable without rmcp and the macros' API churn does not matter. rmcp 3.x names found by compiling: `CallToolRequestParams`, `PaginatedRequestParams`, `CallToolResponse` (from `CallToolResult`), `ContentBlock`, `ServerConfig` (non-exhaustive, built by field assignment).
- **Tool names**: the CLI command path joined with `_` (`ck project get` is `project_get`), because MCP tool names allow only letters, digits, `_` and `-`. Inputs are the same JSON the HTTP API takes (`PatchRequest`, `LockRequest`, `UnlockRequest`); `project_get` takes `{summary?, layer?}` and refuses both (E_REQUEST), like clap's conflict. Unknown fields are refused for `status` and `project_get`.
- **Holder**: MCP `lock`/`unlock` require `holder` (the request type's schema), while the CLI defaults `--holder` to agent. Kept: an explicit holder in a machine call is safer than a default; outputs are identical.
- **Outputs**: success is one text block with the exact line the CLI prints on stdout; failure is a tool result with `isError` and the exact CkError JSON the CLI prints on stderr. An unknown tool name is a JSON-RPC invalid-params error. Caps are the CLI's (shared `ck_server::ops`); `project_get` lists the larger of the summary and layer caps.
- **`ck mcp`** serves stdio until the client closes it; each call finds the studio through `.clipkit/server.json`, so it can start before the studio. Nothing but the protocol goes to stdout.

### Step 5: TS type generation (2026-10-29)
| Crate | Version | Notes |
| --- | --- | --- |
| ts-rs | 12.0.1 | latest per `cargo info` on 2026-10-29; release date PLACEHOLDER (headless run, no web); default `serde-compat` plus `serde-json-impl`; `format` (dprint) left off to keep the build small |
- **ts-rs over specta**: plain derive that reads the serde attributes already on every type (rename_all, tag, flatten, skip_serializing_if, rename), no runtime registry. API found by compiling: `TS::decl(&Config)`, `Config::new().with_large_int("number")`.
- **One file**: `studio/src/generated/types.ts`, built by `ck_server::ts_types::render()` from an explicit list (model, API requests/replies, CkError/ErrorCode, Event, JsonValue). Lives in ck-server because it sees both crates. A test (`every_referenced_type_is_declared`) catches a type left off the list.
- **Integers as `number`**: u64/usize revs, ms and counts are JSON numbers well under 2^53, so `bigint` would mistype them; a test forbids `bigint`.
- **Patch ops** are `Array<JsonValue>` on the TS side (`#[ts(as = "Vec<Value>")]`); json-patch has no TS impl. A plain comment, not a doc comment, so the JSON Schema (and the MCP snapshot) is unchanged.
- **Drift check**: the nextest test `generated_types_match_committed_file` compares the committed file with a fresh render, so `just check` and CI fail on drift; `just types` (CK_WRITE_TYPES=1) rewrites it. Server-only types (ServerInfo, history entries, ops caps) are not exported until the studio needs them.

### Step 6a: studio scaffold and embedding (2026-10-29)
npm versions: newest stable release at least two weeks old per `pnpm view <pkg> time` (registry reachable in this run), written into `studio/package.json` by hand (`pnpm add` is denied headless) and installed with `pnpm install`; `pnpm-lock.yaml` committed.
| Package | Version | Released |
| --- | --- | --- |
| react, react-dom | 19.3.1 | 2026-10-14 |
| @types/react, @types/react-dom | 19.3.0 | 2026-09-09 |
| remotion, @remotion/player | 4.0.535 | 2026-10-12 |
| vite | 8.3.4 | 2026-10-08 |
| @vitejs/plugin-react | 6.1.2 | 2026-10-05 |
| typescript | 7.0.2 | 2026-07-08 |
| prettier | 3.9.10 | 2026-10-10 |
| rust-embed (crate) | 8.13.0 | latest per `cargo info`; release date PLACEHOLDER (cargo shows none; web and the cargo index cache outside the repo were not reachable) |
- **Token handoff**: the studio reads the token from the URL fragment (`#token=...`, never sent to the server), keeps it in sessionStorage and strips it from the address bar. `ck studio` still never prints the token; Phase 3's `--open` opens `url#token=...` from server.json. Until then a human (or the step 7 Playwright test) builds that link from server.json.
- **Static files need no token** (no project data in them) but pass the site guard (Host, Origin). Paths without an extension fall back to index.html; missing files with an extension, `..` segments, unknown `/api/*` paths are E_NOT_FOUND JSON; non-GET/HEAD is 405. Hashed `assets/*` are cached immutable, index.html `no-cache`; every file gets `nosniff` and `no-referrer`. No CSP yet: Remotion's inline styles would need `style-src 'unsafe-inline'`; revisit when the panels land.
- **rust-embed** with `allow_missing`, so ck builds before the studio does (a page then says to run `just studio-build`). Debug builds read `studio/dist` from disk; release builds embed it, so build the studio before `cargo build --release`. Content types are a small hand map, not mime_guess.
- **Live state from snapshots only**: the studio's project comes from `snapshot` events (sent on connect and after each change); no separate initial fetch. Item timing in TS (`studio/src/timeline.ts`) mirrors `ck_core::timeline::span`; the Player duration is at least one frame.
- **Preview shell**: the composition draws each visible layer as a labelled box with `data-ck-layer`; media from proxies and component clips arrive with their phases.
- **Checks**: `just check` now ends with `just studio-check` (prettier check + `tsc --noEmit`, strict); CI installs pnpm (`pnpm/action-setup@v4`, from `packageManager`) and Node 24 (`actions/setup-node@v5`); those action majors are unverified (headless).
- TypeScript 7 (native) typechecks; Vite bundles the config itself, so `vite.config.ts` is outside the tsconfig and no `@types/node` is needed (step 6b adds it for the unit tests).

### Step 6b: studio panels (2026-10-29)
| Package | Version | Released |
| --- | --- | --- |
| @types/node | 24.19.2 | 2026-10-09 (newest 24.x at least two weeks old, `pnpm view`) |
- **Unit tests with `node --test`**, no test framework dependency: Node 24 strips types, so pure modules (`studio/src/state.ts`) run as-is. Such modules use only `import type` for other studio files (erased), and tests import them with the `.ts` extension (`allowImportingTsExtensions`, fine under `noEmit`). `pnpm test` runs from `just studio-check`.
- **Lock liveness in the browser** uses the browser's `Date.now()` against `heartbeatAtMs + ttlMs`. The server runs on the same machine with the same wall clock, and a 1 s ticker re-evaluates expiry. The server stays the authority; the UI only disables controls.
- **Read-only scope** matches the server's lock gate exactly: a layer's controls are disabled while the agent holds a live lock on that layer id. Spec 5.3's global lock for structural changes has no lock id in ck-core yet, so the UI has none either (revisit with Phase 6 layer locks).
- **Edits** are single `replace` ops on name, purpose, `at` and clip `dur`, sent with `baseRev` and the layer's `layerRevs`, so an edit built on an old view is E_PATCH_STALE instead of overwriting. The schema-driven props inspector, drag, trim and split are Phase 6 (spec 5.4).
- **Banner**: shows status, stage or pass name, and the agent's label. Shows "agent stopped" when the agent had locks and all of them lapsed without a release (spec 5.3 crash safety). The model has no agent name yet, so it says "Agent".
- **Ghost cursor and outline** mark the agent's live locked layers plus `agent.activeLayers` (while not idle), on the timeline and on the canvas. The canvas gets them through Player `inputProps`, so the composition stays a pure function of its props. Touched layers flash for 600 ms; timeline items move with a 200 ms CSS transition (watch mode is UI only).
- **Not in 6b**: the Stop button (needs cooperative cancel through the event bus; Phase 3 brings agent events), token usage per feed line (the meter arrives later), CSP (still open).

### Step 7: Playwright e2e (2026-10-29)
| Package | Version | Released |
| --- | --- | --- |
| @playwright/test | 1.64.1 | 2026-10-14 (newest stable at least two weeks old, `pnpm view`) |
- **Chromium only** for now (`pnpm --dir studio exec playwright install chromium`, one time). The studio targets a local desktop browser; more engines can be added as Playwright projects later.
- **One real `ck studio` per test** on a fresh temp folder (`studio/e2e/fixture.ts`), driven by the real debug `ck` binary (`CK_BIN` overrides). No fixture project is copied: Phase 2 has none, and the test builds its layer through `ck project patch`, which is itself the thing under test. Tests are independent, so they run in parallel.
- **Latency measurement**: from just before the CLI process spawn to the layer node appearing in the page (MutationObserver, `Date.now()` on the same machine). This is an upper bound on the patch-to-preview time because it includes process startup.
- **Server refusal from the page**: the refused patch is sent with `fetch` inside the page (studio Origin, session token from sessionStorage), built with the studio's own `fieldPatch`, so it is the same request the inspector would send.
- **`just e2e` is not part of `just check`** (CLAUDE.md lists it separately; it needs a browser and a build). Not yet in CI: adding a job needs the Playwright browser cache and is left for step 8 or Phase 3.
- **Banner** is checked only in the idle state: `/agent` is read-only to patches and no command sets it until Phase 3.

### Step 8: wrap-up (2026-10-29)
- **Release dates checked** against the crates.io API (`https://crates.io/api/v1/crates/<name>/<version>`, fetched with node) for every registry crate in the lock graph (204). Direct crates under two weeks old on 2026-10-29 were pinned back with `=` to the newest release on or before 2026-10-15:

| Crate | Was | Now | Released |
| --- | --- | --- | --- |
| tokio | 1.54.1 (2026-10-21) | 1.53.2 | 2026-10-03 (1.54.0 is yanked) |
| ureq | 3.5.0 (2026-10-21) | 3.4.2 | 2026-09-13 (3.4.3 is also 2026-10-21) |
| rmcp | 3.8.0 (2026-10-26) | 3.5.1 | 2026-10-05 (3.6.0 and 3.7.0 are also newer than 2026-10-15) |

  The other direct crates are old enough: axum 0.8.9 (2026-04-14), getrandom 0.4.3 (2026-06-17), tower 0.5.3 (2026-01-12), tokio-tungstenite 0.29.0 (2026-03-17), futures-util 0.3.34 (2026-08-11), clap 4.6.7 (2026-09-14), ts-rs 12.0.1 (2026-01-31), rust-embed 8.13.0 (2026-10-07). The step 1 to 6a PLACEHOLDER dates above are replaced by these. Still too new, all transitive: cc 1.10.0, find-msvc-tools 0.1.15, libc 0.2.191, rustix 1.1.6, smallvec 1.16.4, uuid 1.29.0, ureq-proto 0.6.5 (2026-10-16 to 2026-10-27). Pinning them needs `cargo update --precise`, which headless runs cannot use (it is on the ask list). Open question.
- **Idle shutdown (spec 5.1)**: `ck studio --idle <seconds>`, default 1800 (30 min), 0 never stops. Activity means a token-authorized API or `/ws` call; static files and refused calls do not count, so a stray local request cannot keep the server alive. An open WebSocket (a studio tab) holds the server up, and closing it restarts the count. On idle the server stops gracefully and removes `server.json`. `idle::Activity` is runtime state, not a wire type, so it has no serde/JsonSchema derives. Time is tokio's monotonic clock (tests pause it), not the session's wall clock.
- **`ck heartbeat` and MCP `heartbeat`**: the route existed without a CLI or MCP twin, so an agent could not renew its locks. Same input as the HTTP body (`holder`, CLI default agent) and the same 4 KB cap and trimming as lock/unlock.
- **CSP**: every studio file response carries `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; media-src 'self' data: blob:; font-src 'self' data:; connect-src 'self'; worker-src 'self' blob:; object-src 'none'; base-uri 'none'; form-action 'self'; frame-ancestors 'none'` plus `X-Frame-Options: DENY`. Inline styles are needed by React and Remotion `style` attributes; no `unsafe-eval` or inline scripts. The e2e suite records `securitypolicyviolation` events and requires none. The first run caught a `media-src data:` violation from the Player, so `data:` is allowed for media.
- **CI e2e job** added to `.github/workflows/ci.yml`: install studio deps, `playwright install --with-deps chromium`, `just e2e`. No browser cache yet (simplest option). It is unverified until the first push.
- **Studio open time (spec 1.2, under 3 s)** is measured in the e2e: from `page.goto` to the connected timeline.
- **Not changed (review findings recorded as open)**: status/summary reads over the cap are refused instead of paged; heartbeat and no-op unlock still fsync and publish; `--holder human` is accepted from CLI/MCP (role trust boundary, already open); the second-studio check has a check-then-bind gap; no cap on concurrent WebSockets; `trimmed` can still refuse if the fixed fields alone exceed the cap (not reachable with today's fields).
## Phase 1 (2026-10-28)

### Crate versions (crates.io release dates checked 2026-10-28; all at least two weeks old)
| Crate | Version | Released | Notes |
| --- | --- | --- | --- |
| serde | 1.0.229 | 2026-07-18 | `derive` |
| serde_json | 1.0.154 | 2026-10-11 | `float_roundtrip`: without it a proptest found f64 values that change by one ULP across save and load |
| json-patch | 4.2.0 | 2026-04-30 | `schemars` feature (depends on schemars 1.x) so patch ops derive JsonSchema |
| schemars | 1.2.3 | 2026-10-12 | |
| thiserror | 2.0.21 | 2026-09-23 | |
| tracing | 0.1.44 | 2025-12-18 | |
| tracing-subscriber | 0.3.23 | 2026-03-13 | `json`, `env-filter`; ck-cli only |
| proptest | 1.12.0 | 2026-10-10 | dev |
| insta | 1.49.0 | 2026-10-03 | dev, `json` |
| tempfile | 3.27.0 | 2026-03-11 | dev |

### Model choices where spec 3.2 is silent
- Items are a common struct plus a body tagged by `type` (footage, clip, voice, audio, caption); `rev` per item is engine-set.
- Footage and audio `src` are source ids and are checked against `sources`. Caption `ref` names a source or voice slot.
- `plan`, `derived` and `branches` stay untyped JSON until Phases 4, 8 and 9 define them.
- Voice slot status values: empty, pending, ready, failed. Brief `effort` and `licenseMode` are strings until their phases.
- Locks live in the project (spec 3.2) and carry holder (agent or human), heartbeat and ttl.
- Undo and redo are new revisions (approved by the owner); default lock ttl 30 s (approved).

### Engine choices (Phase 1)
- **Content vs engine state**: diffs and undo work on project content with `rev`, item `rev`, `locks`, `pipeline` and `agent` removed. Patches may not touch those fields (E_PATCH_INVALID); the engine sets them.
- **Lock ids beyond layers**: the spec locks "touched layers" but is silent on non-layer content. Simplest safe rule: an item id when the item is added, removed, changed or moved to another track; a track id when the track's fields or the relative order of its items change; `@project` for any other content (settings, brief, sources, voice slots, comments, tokens, plan). Ids may not start with `@`. The agent must hold every touched id; humans are refused on agent-held ids. (Found by the Phase 1 rust-reviewer: without this, agent edits outside items skipped the lock gate.)
- **Locks on ids that do not exist yet** are allowed, so the agent can lock a layer before adding it. Lock changes are not revisions; they are saved inside project.json.
- **A patch that changes nothing** is refused (E_PATCH_INVALID) so history holds no empty revisions.
- **Stale patches** are refused whenever `baseRev` is behind (no automatic rebase); the error lists items changed since `baseRev` plus any `layerRevs` mismatches.
- **Storage**: `history/base.json` is the replay starting point; history must run contiguously from base rev + 1 to the project rev, and each file name must equal the rev inside. Temp files are `.<name>.tmp-<pid>` and only those are removed on load.
- **Log field names** are camelCase as spec 10.4 writes them (traceId, runId, turnId, layerId, plus layerIds for multi-layer events). `ck` logs JSON to stderr only when `CK_LOG` is set.
- **Stage machine**: VERIFY may send work back to any pass; ITERATE always returns to VERIFY; leaving BRIEF needs a confirmed or auto brief. Operation-to-stage table in `ck-pipeline/src/stage.rs`.
- **Retry gate**: a call is the op name plus its arguments with keys sorted; refused after 2 failures, reset by a success.

### Autonomous build loop (2026-10-28)
- Commits and PRs carry no Claude attribution (owner request): `.claude/settings.json` sets `attribution.commit` and `attribution.pr` to "" (the `attribution` key replaces the deprecated `includeCoAuthoredBy`; code.claude.com settings reference).
- `scripts/autobuild.sh` pushes `main` to origin after a run that made a commit and passed `just check` (plain push, never forced; `PUSH=0` turns it off). The agent never pushes.
- Dry run (MAX_RUNS=1, 2026-10-28): success, 1 commit (Phase 2 steps 1-2), `just check` green, 30 permission denials: 18 WebFetch + 1 WebSearch (crate version lookups; no allow rule), 11 Bash (paths outside the repo, curl, shell loops, python3/sed edits, cd-prefixed compounds). The run reported 102 turns with `--max-turns 80`, so the flag may not cap turns as expected.
- `scripts/autobuild.sh` (owner-supplied) runs `claude -p` sessions with `scripts/autobuild-prompt.md`. It reads `STATUS:` and `PHASE:` from the top of PROGRESS.md.
- Checked against the installed CLI (2.1.312): `-p`, `--output-format`, `--permission-mode` are documented in `claude --help`; `--max-turns` is not listed there but is accepted (an unknown flag errors with "unknown option", this one does not).
- The prompt file is the owner's text plus a short "Loop contract" section: keep the STATUS/PHASE lines current, skip the phase-runner approval wait in autonomous runs, and recover an interrupted run's uncommitted changes first.
- `--permission-mode acceptEdits` approves file edits only; Bash runs only what `.claude/settings.json` allows, and anything else is denied in `-p` mode rather than prompting. The owner approved adding git log/show/rev-parse, ls, wc and jq to the allow list (git status/diff were already there) and the autonomous-mode note in CLAUDE.md and the phase-runner skill. git push, dependency installs (`cargo add/install/update`, `pnpm add`, `npm install`, `npx`) stay "ask", which headless runs treat as denied; `.env` reads stay denied.

## Phase 0 (2026-10-28)

### Repository
- **Reuse the existing GitHub repo** (`Sanidhya14321/clipkit-studio`, local folder `Hackcbs-Project`)
  at the owner's request, to keep contributors. The previous Codex-built TypeScript implementation is
  preserved on the branch `codex-archive` (commit `a53a030`, pushed to GitHub until all phases are done; including all of its uncommitted
  work) and removed from `main`. Its untracked, gitignored leftovers (`.clipkit/`, `node_modules/`,
  `artifacts/`, build output, about 5 GB) were moved to `~/Desktop/clipkit-codex-leftovers/`, not deleted.
- Kept `LICENSE` (MIT, "ClipKit Studio contributors").
- The repo root is the workspace root (spec 3.1 shows a `clipkit/` folder; that folder is this repo).

### Claude Code formats (verified 2026-10-28 in the official docs)
Sources: code.claude.com/docs/en/memory, /skills, /sub-agents, /hooks, /hooks-guide, /permissions, /settings.
- **Memory**: project memory is `./CLAUDE.md` (or `./.claude/CLAUDE.md`); `CLAUDE.local.md` is personal
  and gitignored. `@path` imports exist (max depth four) but do not reduce context cost, so CLAUDE.md
  uses plain pointers instead. Docs target under 200 lines; this project keeps it under 120 (spec 11.1).
- **Skills**: `.claude/skills/<name>/SKILL.md`, YAML frontmatter then Markdown. All fields optional;
  we set `name` and `description` (description + `when_to_use` are truncated at 1,536 chars in the
  listing). Supporting files sit in the skill folder and are linked with relative Markdown links.
- **Subagents**: `.claude/agents/<name>.md`; required `name` and `description`; optional `tools`
  (comma-separated string or YAML list), `disallowedTools`, `model` (`haiku`, `sonnet`, `opus`,
  `fable`, a full model id, or `inherit`), and more. Unknown fields are ignored.
  - Pattern-scoped entries such as `Bash(git diff *)` inside an agent's `tools` were **not verified**,
    so agents list plain tool names and use `disallowedTools` (Edit, Write, NotebookEdit) for read-only
    roles; the prompt restricts Bash to read-only git commands for rust-reviewer.
  - Models: `haiku` for test-runner and researcher (routine), `sonnet` for spec-checker and rust-reviewer.
- **Hooks** (in `.claude/settings.json`): `hooks.<Event>[] = {matcher, hooks: [{type: "command",
  command, timeout}]}`. A matcher of only letters/digits/`|` is an exact list (`Write|Edit`).
  stdin JSON carries `tool_name`, `tool_input.file_path` / `tool_input.command`, `cwd`; Stop carries
  `stop_hook_active`. Exit 0 = ok; exit 2 = block, stderr goes to Claude; other = non-blocking error.
  Stop and PostToolUse can return `{"decision":"block","reason":...}`. `$CLAUDE_PROJECT_DIR` is the
  project root. Default command timeout is 600 s, so every hook sets its own.
  - The harness also caps consecutive Stop continuations at 8 (`CLAUDE_CODE_STOP_HOOK_BLOCK_CAP`);
    our Stop hook additionally exits 0 whenever `stop_hook_active` is true, so it blocks at most once.
- **Permissions**: evaluated deny, then ask, then allow; first match wins; list keys merge across
  settings files. `Bash(cargo *)` (space form) matches `cargo` and `cargo build` but not `cargofoo`.
  `Read(./.env)` style path rules are documented; `**` globs in Read rules were **not verified** with
  an exact example, so the guard-edits hook and `.gitignore` back them up.
  `$schema`: `https://json.schemastore.org/claude-code-settings.json`.

### Build setup choices
- **Hook scripts** live in `.claude/hooks/*.sh` (bash + jq) and are tested by `tests/hooks/run.sh`
  with recorded payloads (exit code, output, under 2 s). CI runs it inside `just check`.
- **Stop hook** skips instantly when no Rust-related file changed, so docs-only turns stay fast.
- **guard-bash** is a heuristic net, not a sandbox; permissions are the real boundary. It checks line
  by line so heredoc bodies (for example a `.gitignore` listing `.env`) do not trigger it.
- **Studio formatter hook** uses the package's own `node_modules/.bin/prettier` once Phase 2 installs it;
  until then it does nothing.
- **`.env.example`** stays editable; every other `.env*` file is blocked from edits and reads.
- **Asset quarantine folder** is `assets/quarantine/` (spec 3.1 names no folder).
- **Ask** (not allow) for `cargo add/install/update`, `pnpm add/update/dlx`, `npm install`, `npx` and
  `git push`: these are the "network installs of new dependencies" from spec 11.6. Reads and edits
  outside the repo already prompt by default, so no extra rule is needed.
- **Workspace lints**: `unsafe_code = forbid`, `clippy::unwrap_used = deny` (tests allowed via
  `clippy.toml`), `dbg_macro = deny`.
- **No dependencies in Phase 0**: crates are empty skeletons. Each phase adds the crates it needs,
  checks the current version, and records it below.
- **Fixtures** are generated by `fixtures/generate.sh` (bash, ffmpeg argument arrays) into the gitignored
  `fixtures/generated/`, with `ground-truth.json`. The frame-counter overlay needs ffmpeg built with
  `drawtext`; without it the video is still generated and `ground-truth.json` says `"overlay": false`.

### Tool versions (checked 2026-10-28)
| Tool | Version | Note |
| --- | --- | --- |
| Rust toolchain | 1.99.1 stable (2026-10-13) | pinned in `rust-toolchain.toml`; edition 2024, resolver 3 |
| just | installed via `cargo install --locked` | version in VERIFICATION.md |
| cargo-nextest | installed via `cargo install --locked` | version in VERIFICATION.md |
| actions/checkout | v7 (latest v7.0.1) | `gh api .../releases/latest` |
| Swatinem/rust-cache | v2 (latest v2.9.3) | |
| taiki-e/install-action | v2 (latest v2.88.0) | installs just and cargo-nextest in CI |
| jq | 1.8.1 local | hooks depend on it |
| ffmpeg | not installed locally; Ubuntu candidate 8.0.1 | CI installs it from apt |
| Node | 24.21.0 local | studio and render worker arrive in Phases 2 and 9 |

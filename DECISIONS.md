# Decisions

Newest first within each section. Each entry: what, why, and the source when it is a verified fact.

## Phase 3 (2026-10-29)

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

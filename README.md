# ClipKit Studio

A Rust-powered video editing and generation plugin for coding agents, with a live localhost studio
where you watch the agent edit and take over when it stops.

The agent works through `ck` (a CLI and an MCP server with the same tools). Every change is a
patch on a non-destructive project you can undo, comment on and edit yourself in the studio.

**Status:** Phases 0 to 8 are built; Phase 9 (polish and proof) is in progress. See
`PROGRESS.md` for where the build stands, `VERIFICATION.md` for evidence and `docs/spec/INDEX.md`
for the specification.

## Requirements
- `ffmpeg` and `ffprobe` on `PATH` (import, proxies, cuts, loudness, fast export).
- Node 24 and pnpm (the studio build and the render worker). Rendering component layers uses
  Remotion, which downloads Chrome Headless Shell on first render.
- `git` (fetching sources for the asset hub).
- Building from source: Rust (the version in `rust-toolchain.toml` installs through rustup),
  `just`, `cargo-nextest`, `jq`.

## Install (from source)
```
just studio-build                      # the studio is embedded in the binary
cargo install --path crates/ck-cli     # the single `ck` binary
just worker                            # render worker: install and bundle (needs Node and pnpm)
ck init                                # set up Claude Code, Codex, Cursor or any MCP client
ck uninstall                           # remove everything `ck init` added
```
`ck init` shows what it will write and asks first. Packaged installers are not built yet.

## Demo
```
mkdir demo && cd demo
ck demo reset          # seeds a 15 s project: confirmed brief, three scenes, voice script, a comment
ck studio              # open the studio from the printed link
ck export              # renders through the worker to exports/demo-source.mp4
ck eval exports/demo-source.mp4
```
`ck demo reset` rebuilds the seeded project in well under a second and refuses to replace a
project that is not the demo unless you pass `--force`.

## What the agent and you can do
- Generate: brief, recipes with typed slots, design-token looks, layout solver, quality gates.
- Edit footage: import, proxies, transcript, cut suggestions, delete words to cut, fast export.
- Interact: comments pinned to moments, ranges, layers or elements; hover-select; tuning chips;
  `ck context`; the editor is read-only only for what the agent has locked.
- Voice: voice slots, record-along with a teleprompter, takes, captions from take timing,
  push-to-talk commands, `ck ask`.
- Assets: a Source Hub that vets sources without running their code, cards and search with a
  retrieval budget, license modes and a license report.
- Debug: `ck replay`, `ck bisect`, `ck trace bundle` (redacted), `ck render check`.

## Voice privacy
- Recordings you make in the studio stay in the project folder (`.clipkit/cache/`).
- Fish Audio (text to speech, transcription, voice cloning) is not connected yet: those calls
  are stubs until the owner adds a key. Once connected, text and audio sent for speech,
  transcription or cloning go to Fish Audio under its terms.
- Cloning uses only takes recorded in the studio, and only after you tick the consent box on
  that take. The agent cannot give consent for you; the server refuses it.
- Push-to-talk uses your browser's speech recognition when it has one. In Chrome that sends
  your audio to Google's speech service. The typed command field sends nothing anywhere.

## Remotion licensing
The studio preview and the render worker use [Remotion](https://www.remotion.dev). Remotion is
not under this repository's license: it has its own license, and companies above a certain size
need a company license. Read https://remotion.dev/license before using ClipKit commercially.
The export license report lists the Remotion runtime separately from every asset.

## Asset licenses
- Taste files, looks, recipes and core clips in this repository are original ClipKit work under
  this repository's license.
- Third-party sources live in the Source Hub (`sources.yaml`, `sources.lock`). Each license is
  read from the source's own license text during vetting; unverified sources are never
  searchable. Ingested cards keep the source's LICENSE beside them.
- The brief's license mode (commercial-safe by default) decides which assets search, derive
  and export allow. `ck license report` (and every export) lists each asset used with its
  license, attribution line and notes, such as trademark notes for brand icons.
- Raw third-party sources in `assets/quarantine/` are not committed; they are fetched again from
  the commit pinned in `sources.lock`.

## Development
```
just check      # fmt, clippy, tests, hook tests, studio checks
just test       # full suite
just e2e        # Playwright browser tests
just fixtures   # synthetic test media + ground truth in fixtures/generated/
just worker     # render worker install and bundle
```

## License
MIT, see `LICENSE` (ClipKit's own code and content). Remotion and third-party assets are under
their own licenses, as above.

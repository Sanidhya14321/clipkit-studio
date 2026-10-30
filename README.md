# ClipKit Studio

A Rust-powered video editing and generation plugin for coding agents, with a live localhost studio
where you watch the agent edit and take over when it stops.

The agent works through `ck` (a CLI and an MCP server with the same tools). Every change is a
patch on a non-destructive project you can undo, comment on and edit yourself in the studio.

**Status:** early and in active development. The studio editor (timeline move, trim and split),
the Edit Decision Plan and the export queue work; the benchmark against a vanilla agent has not
been run. See `PROGRESS.md` for where the build stands, `VERIFICATION.md` for evidence and
`docs/spec/INDEX.md` for the specification.

## Install
```
curl -fsSL https://raw.githubusercontent.com/Sanidhya14321/clipkit-studio/main/install.sh | sh
```
This downloads the release for your machine (macOS on Apple silicon, or Linux x86-64), checks its
checksum, installs `ck` and the render worker to `~/.clipkit`, and offers `ck init`, which shows
what it will write to Claude Code, Codex, Cursor or any MCP client and asks first. `ck uninstall`
removes what `ck init` added; delete `~/.clipkit` to remove the rest.

You also need `ffmpeg`, `ffprobe`, Node 24 and `git` on `PATH`; the installer names any that are
missing. Remotion downloads Chrome Headless Shell on the first render.

## How a video gets made
EDIT: `ck import`, `ck transcribe`, `ck content map`, `ck grid build --source <id>`. GENERATE:
`ck grid build --dur <seconds>`, fast by default. Then both paths use `ck plan set`,
`ck plan candidates` and `ck plan expand`, followed by `ck verify` and `ck export` (`ck export --queue` returns at once; `ck jobs` shows progress). The agent
decides per segment; code does the frame math. Changing one part with
`ck plan expand --segment <id>` touches only that segment.

## Install from source
Needs Rust (the version in `rust-toolchain.toml`), `just`, Node 24 and pnpm; tests also need
`cargo-nextest` and `jq`.
```
just studio-build                      # the studio is embedded in the binary
cargo install --path crates/ck-cli     # the single `ck` binary
just worker                            # render worker: install and bundle
ck init
```

## Releasing
Push a `v*` tag. `.github/workflows/release.yml` builds both platforms, installs each package as a
test and publishes a GitHub release.

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

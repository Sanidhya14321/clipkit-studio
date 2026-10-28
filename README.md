# ClipKit Studio

A Rust-powered video editing and generation plugin for coding agents, with a live localhost studio
where you watch the agent edit and take over when it stops.

**Status: Phase 0 (build setup).** No product features yet. See `PROGRESS.md` for where the build
stands and `docs/spec/INDEX.md` for the specification.

The earlier TypeScript implementation is preserved on the `codex-archive` branch.

## Requirements (development)
- Rust (the version in `rust-toolchain.toml` installs automatically through rustup)
- `just`, `cargo-nextest`, `jq`
- `ffmpeg` and `ffprobe` (for fixtures and the media engine)

## Commands
```
just check      # fmt, clippy, tests, hook tests
just fixtures   # synthetic test media + ground truth in fixtures/generated/
```

Install, demo, voice privacy, Remotion licensing and asset licenses are documented as those phases land.

## License
MIT, see `LICENSE`.

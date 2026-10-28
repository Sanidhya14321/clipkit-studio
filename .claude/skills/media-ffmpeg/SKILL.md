---
name: media-ffmpeg
description: Use when invoking ffmpeg or ffprobe (ck-media, fixtures, export). Safe invocation as argument arrays, probing, proxies, loudness, filter-graph export and the fixture media.
---

# Media and ffmpeg

## Invocation
- Always argument arrays: `Command::new("ffmpeg").args([...])` or `tokio::process::Command`.
  Never `sh -c` and never a format!-ed command string. Paths are separate args, never interpolated.
- Always: `-hide_banner -nostdin -loglevel error -y` (and `-progress pipe:1` when reporting progress).
- Put user-supplied paths after `-i` or as outputs only; reject paths starting with `-`.
- Capture stderr to the run's trace folder; surface one line plus an error code (`E_MEDIA_*`).
- Timeouts on every call; kill the child on cancel.

## Probing
- `ffprobe -v error -show_streams -show_format -of json FILE`, parse with serde. Never parse
  human-readable output. Durations are seconds as f64.

## Proxies
- Proxy = low-res H.264 with frequent keyframes for fast seeking (for example `-g` = fps).
  Keyed by content hash in `.clipkit/cache/`. Source media is never modified.

## Loudness
- Two-pass `loudnorm`: measure (`print_format=json`), then apply with measured values.
  Silence detection: `silencedetect=noise=-40dB:d=0.3`, parsed from stderr lines.

## Fast export
- Build one `-filter_complex` graph from the project (trim/setpts, concat, overlays, captions,
  amix). Build it as a typed graph and render to a string in one place, with tests on the string.

## Fixtures
- `just fixtures` runs `fixtures/generate.sh` -> `fixtures/generated/` (gitignored) with
  `ground-truth.json`: frame counter and timecode video, tone bursts and silences at known
  times, scene changes every 2 s, images in three aspects, a brand kit and a fake transcript.
- Tests read times from ground-truth.json, never hard-code them. Never read large media into context.

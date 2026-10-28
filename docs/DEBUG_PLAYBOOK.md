# Debugging ClipKit

Start with `pnpm ck doctor` (installed package: `ck doctor`). Keep the run ID from the response or Studio Systems → replay → Flight recorder. Existing product errors retain their original code and add `debugCode` and `hint`.

## Commands

```sh
ck trace list
ck trace bundle RUN_ID
ck replay RUN_ID
ck bisect RUN_ID --check schema
ck repro RUN_ID --layers LAYER_ID
ck fixtures generate /tmp/clipkit-fixtures
ck eval /path/to/result.mp4 --project /path/to/project.json
ck bench matrix bench/update4.json --out /tmp/clipkit-results
ck bench report /tmp/clipkit-results
```

Use `ck --help` for the installed version's flags. Replay rebuilds anonymized revisioned JSON without an agent or provider call. Bisect scans revisions in order, including non-monotonic failures. Repro reduces selected layers and their referenced sources; it does not package original media or voice samples. Regenerate synthetic fixtures separately. Source/asset execution is still subject to existing verification and license gates.

ZIPs include only bounded JSON, JSONL and documentation files. They exclude media, environment files, credentials and symlinks. Default recording anonymizes user copy and paths. Synthetic mode is an explicit test-only option. A redacted replay is a diagnostic project, not a portable original film. Inspect a bundle before sharing it.

## Symptom → first step

| Symptom | First command or inspection |
| --- | --- |
| Studio does not update | `ck doctor`; compare WebSocket connect/drop events, latest recorded rev and Studio rev. |
| Agent misses the pipeline/context | Check adapter registration with doctor, then the agent transcript and installed instruction size. |
| Agent waits indefinitely | `ck pipeline status`; inspect pending brief/ask events and the driver's trace ID. |
| Wrong layers change | `ck bisect RUN_ID --check schema`, then inspect the offending patch and selected layer history. Schema checking alone cannot certify edit scope; compare the recorded layer IDs. |
| Export differs from preview | Inspect source/proxy hashes and render cache events; compare decoded frames at three fixed timestamps. |
| Asset render differs across frames | Repeat deterministic frame tests, inspect source pin and grade. Keep unverified sources quarantined. |
| Voice missing or wrong | Check provider mode, cassette key, active take, word timestamps and measured loudness. |
| B performs worse than A0 | Preserve the paired transcripts and run C1–C5 with the same protocol; do not tune a single arm's prompt. |
| Baseline unexpectedly strong | Inspect namespace baseline-clean receipt, mounted files/configuration and evaluator limitations. |

## Stable diagnostic codes

| Code | Likely cause | First step |
| --- | --- | --- |
| E_ENV | Missing binary, unavailable port, disk, credentials or provider quota | `ck doctor` |
| E_SCHEMA | Invalid command or project input | Check CLI/MCP input schema and rejected field |
| E_PATCH_INVALID | Invalid patch or replay hash | Inspect rejected operations / recorded patches |
| E_PATCH_STALE | Revision conflict | Read current affected layer revisions |
| E_LOCK_HELD | Another turn owns edit scope | `ck context`; wait or cancel owner |
| E_LOCK_TIMEOUT | Expired lease | Acquire fresh lease and reread layers |
| E_ASSET_NOT_FOUND | Disabled/missing asset or media | Search enabled catalog and registered media |
| E_ASSET_LICENSE | Missing or incompatible rights | `ck license report`; read original evidence |
| E_SOURCE_VET | Unverified/quarantined source | `ck sources vet`; never infer owner approval |
| E_DETERMINISM | Time/randomness-dependent output | Repeat/reverse-order frame check |
| E_MEDIA_PROBE | Invalid media or probe executable | `ck doctor`; probe synthetic fixture |
| E_TRANSCRIBE | Failed ASR or missing word times | Inspect cassette/provider and transcript |
| E_VOICE_PROVIDER | Provider unavailable or rejected request | Inspect response metadata; use replay fixture |
| E_RENDER | Browser/composition/frame failure | Inspect render event and first failed frame |
| E_EXPORT | Export job/container failure | Inspect job, rights and renderer events |
| E_BRIEF_TIMEOUT | Scripted answer did not arrive | Inspect pending brief and event cursor |
| E_AGENT_TIMEOUT | Wall/token budget exceeded | Inspect last completed pass; never estimate missing tokens |
| E_SANDBOX | Isolation or clean-baseline check failed | Repair environment; preserve attempt as invalid |

No retry may silently replace a failed attempt. Preserve all attempt traces, including timeouts. The agent instruction limits identical retries to two. A last-five test history with any failure is quarantined with an owner and failed trace links; it cannot be silently dropped to obtain a release pass.

Independent operator reproduction and timed playbook evaluation are separate acceptance evidence; automated replay is not a substitute.

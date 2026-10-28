# Update 4 audit — V-0

28 October 2026 (Asia/Kolkata). User authorized cache cleanup plus review and implementation of the supplied E2E/debug-harness update. This phase changes documentation only. Preserve revision-50 root project/checksum, existing compact.ts and LibraryPanel.tsx edits, original source/media and all four unverified Source Hub quarantine pins. Clearing cache means regenerable build/render/test caches, not deleting the cache feature, evidence, paid voice takes, project history or pinned sources.

Reviewed DECISIONS.md, VERIFICATION.md, Update 1–3 audits, tests/scripts/CI, CLI/MCP, runner, store/leases, media/render, replay/benchmark, Source Hub, and Studio browser coverage.

| Requirement | Existing evidence | Gap / implementation |
| --- | --- | --- |
| Unit/integration | 177 tests / 30 files; source/licensing/locks/retrieval/patch/revert/real media/exports | Add systematic debug, contracts, privacy, fault and fairness tests. |
| Browser | 8 Playwright flows; six viewports, comments/ranges/undo, brief, sources | Trace-correlated scripted steps, logs/screenshots, 3 repeated runs; add explicit hover/voice/concurrency/export steps. |
| Real agents | Real child-process app runner, older one-pair observations | No clean repeated A0/A1/B matrix S1–S4. Implement isolated runner and record actual failures/usage; no fixture results as live evidence. |
| Logging | Activity ring; durable bounded TurnRecorder; journal; persistent StudioEvents | No unified trace/run IDs across CLI/MCP/HTTP/patch/locks/render/ffmpeg/WS. Add JSONL recorder, redaction and request correlation. |
| Codes / doctor | Existing product codes, dependency probe/basic CLI doctor | Add stable E_* mappings/hints without removing old codes; richer diagnostics and baseline proof. |
| Fixtures/cassettes | Synthetic tone/test video; mocked provider responses | Single deterministic ground-truth fixture generator; record/replay/live cassette transport; no hidden network in replay. |
| Replay/bisect/repro | Turn metadata export and immutable versions; snapshotAt/three-way history | Add actual revision patch reconstruction, first-failure search, sanitized minimal repro ZIP with hashes. |
| Evaluator | Quality gates, ffprobe/loudness helpers, sparse pixel checks | Output-only media metrics, explicit uncertain OCR/layout, ground-truth scope/cut/source checks, comparable JSON scorecards. |
| Benchmark | Operator import/paired explicit drivers; ten-case report and blind A/B UI | Add 15 scenarios, A0/A1/B and C1–C5 config, same information/usage source, repetitions/ranges/pass rates, regression/flaky gates. |
| Isolation / install | Project adapter installer and packed install smoke | Fresh temp home/workspace, sanitized environment/PATH, clean-baseline contamination proof, scoped adapter uninstall receipts. |
| CI / chaos / cross matrix | Assets workflow; targeted race/resume/stale tests | Fast/nightly/debug artifacts and repeatable fault/flaky gates; unsupported platform/live inputs stay explicit. |
| Earlier manual-only gaps | Live mic/voice clone, real stumbles/brand kit, final mix, real human aesthetics/A-B, independent NLE readers, sandboxed third-party previews | Keep explicit; use synthetic/cassette checks without falsely certifying these. Unverified sources stay quarantined. |

Cache inventory: node_modules/.cache/webpack ~2.1GB; test reports/screenshots; regenerable layer/footage views. Generated packaged artifacts are not caches needed for source cleanup; historical A/B, quarantine, registered media, transcripts and paid voice outputs must survive. Clean only enumerated safe locations after checking idle server; record removed bytes and preserved checksum.

Open inputs: real speech/consented clone/brand fixtures and independent human acceptance are not supplied. Agent authentication/model availability and current CLI contracts must be checked before measured runs. The evaluator/release gate must report no-go or unmeasured when these are absent, not manufacture passing evidence.

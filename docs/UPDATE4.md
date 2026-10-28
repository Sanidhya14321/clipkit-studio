# Update 4: verification and debug harness

The saved film is preserved. The added harness records revisions, redacts diagnostic data, generates deterministic fixtures, runs isolated real agents and reports missing measurements explicitly. Read [DEBUG_PLAYBOOK.md](DEBUG_PLAYBOOK.md) for commands and error recovery. The demo release gate remains NO_GO until all required evidence exists.

## Running locally

```sh
pnpm typecheck
pnpm test --maxWorkers=2
pnpm build
pnpm exec playwright test --repeat-each=3
pnpm ck fixtures generate .clipkit/update4/fixtures
pnpm exec tsx scripts/debug-render-preflight.ts
pnpm exec tsx scripts/debug-plugin-preflight.ts
pnpm exec tsx scripts/run-update4.ts
pnpm ck bench report .clipkit/update4/results
pnpm exec tsx scripts/debug-gate.ts --current .clipkit/update4/results/summary.json
```

The last command deliberately exits nonzero for absent acceptance evidence. A future accepted comparison supplies `--baseline last-good-summary.json` and `--acceptance receipts.json`. Receipt keys are priorP0, threeCleanDemos, liveProvider, independentRepro, blindHumanRatings, offlineFallback, securityContracts, noCriticalBugs and noUnexplainedFlakes; values are paths to reviewed evidence. Creating a file does not replace the review it records.

## Frozen experiment

`bench/update4.json` selects Codex 0.162.1, model gpt-6.1-sol, low reasoning, A0/A1/B, S1–S4 and three repetitions. Each agent has a common 120-second active budget, 600,000 reported-token ceiling and two identical scripted follow-up requests. Initial protocol uses zero automatic retries to bound model spend; all first attempts remain in the comparison. The reusable runner supports up to two diagnostic retries, records every attempt and persists last-five flaky history with an owner. Retries never replace first-attempt comparison statistics. Token limits are checked at usage events. Codex resume totals are cumulative and counted once, confirmed by a real three-turn preflight. Interrupted sessions retain known reported usage with complete=false; partial totals are excluded from comparative token statistics.

Homes, workspaces, caches and PID/filesystem namespaces start fresh using Linux bubblewrap. Common dependencies/browser are read-only mounts; each baseline records an in-namespace clean receipt before agent execution. It checks skills, configurations, PATH, ClipKit environment and Studio visibility, boots the default Chrome command and verifies a writable private cache. The Codex client automatically seeds its own 49 packaged system-skill files. Their exact 0.162.1 content hashes are pinned and verified on resume; modified defaults or any extra skills invalidate the baseline. These inherent CLI defaults are shared by all arms. A0 receives no ClipKit runtime, instructions, skill or MCP. A1 may mount only a verified original official-guidance pin, with original rights evidence and unchanged inventory. A path flag never grants source approval. B receives packaged CLI/MCP, agent instructions, skill and its own host-side Studio with fixture-only media. The host Studio runs outside the agent namespace; this architecture is recorded rather than described as a fully containerized server.

The provider network is shared for authentication/model requests; there is no egress allowlist. Only the selected temporary home receives authentication, which is removed on teardown and never bundled. Do not run arbitrary unverified code in this benchmark. This does not certify a third-party preview sandbox.

All arms receive the same facts, fixtures, starter Remotion source and feedback text. B additionally has a confirmed synthetic brief and corresponding imported fixture/selected scene. Setup time is reported separately from the common active budget; reported wall time includes output evaluation before teardown. Unrequested output resolution is not a quality penalty: aspect is checked for S1–S3, while S4 must preserve its explicit starter dimensions. First-preview measurement is first appearance of `result.mp4`, not subjective readiness or first browser paint. The equal feedback currently uses agent follow-up prompts in every arm; B-specific browser comment/brief/voice feedback delivery is **not yet automated**. S5–S15 have catalog entries and existing unit/integration/browser coverage where available; this real-agent runner rejects them as catalog-only until dedicated drivers exist.

Preflight failures with missing browser mounts/aliases or CLI-default seeding misclassification were retained as invalid environment evidence in `.clipkit/update4/preflight-invalid-browser-*`. Their prompts/results do not establish quality comparisons. No arm prompt, model, budget or target criteria is tuned from those setup failures. A separate offline render preflight now proves bundling and browser rendering; an MCP preflight proves packaged tool connectivity.

## Measurement and limits

Results are JSONL with agent version/model, protocol/environment key, flags, limits, baseline/health receipts, errors, tokens, timing, outputs and trace links. The common key includes fixture manifest, scenario text/answers/feedback/criteria, runtime hash, model/version, Node, OS/architecture and machine class. Each trace retains the complete environment receipt separately. Input/output/cached tokens come from the agent's own usage events. Codex price is unmeasured; Claude client-estimated cost is kept distinct. Timeout tokens are never inferred from the internal ClipKit retrieval meter.

Scorecards use decoded output probing/loudness, optional source bounds/cuts and immutable original fixture hashes. Missing OCR, caption alignment, frame-only scope, render-scope or human evidence is unmeasured. A required missing criterion prevents scenario acceptance. Source declarations are supplemental and cannot establish output visual correctness. Freeze/repeated-frame/black diagnostics alone are not aesthetic failures; intentional stills/black frames remain possible. The evaluator is an initial technical evaluator, **not the complete independent acceptance oracle** requested by the spec.

Reports include first-attempt medians/ranges, valid counts, measured token counts, pass rates and an SVG token chart. Blind human A/B remains the existing separate randomized Studio rating workflow; no ratings are synthesized. Agent-in-loop ablation results require running the same controlled protocol; feature switches alone are not findings.

## Evidence still required

A1's original remotion-dev/skills snapshot `30d4553ecd46fbc5212fd0ff006ae43af522158b` has no original license file and is quarantined under the owner's policy. Claude is installed but unauthenticated. Real stumbles, a consented clone sample, a real brand kit, live provider acceptance, physical microphone/dictation, independent reproduction and timed human playbook evaluation are absent. Full live input/output parity for every legacy CLI/MCP command, all chaos cases, performance/long-session metrics and S5–S15 agent drivers remain partial or pending. Cross-OS/Node/agent matrix is deferred at the specified cut line.

Fast CI runs schema/unit/integration/assets/type checks. Nightly/manual CI runs the browser suite three times and uploads synthetic evidence; failures create bounded redacted trace ZIPs. Authenticated model experiments are intentionally on demand via the local runner. CI configuration here is reviewed locally; a successful hosted CI run is separate evidence.

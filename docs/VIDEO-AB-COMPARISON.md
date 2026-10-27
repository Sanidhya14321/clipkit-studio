# Fifteen-second same-prompt comparison

Two actual Codex generations use the **same creative prompt byte-for-byte**, gpt-6.1-sol, medium effort, 1080×1920, 30 fps and 450 frames. Both use the same renderer and technical scene validation. Decoded audio hashes match: an identical original rhythm track was added after generation. No stock footage or fabricated product capture is involved.

The baseline receives a minimal renderer/output contract and no ClipKit skills, MCP or motion kit. The plugin version receives its direction pipeline, relevant skills, visual reference and frame-driven motion helpers. System/workflow context intentionally differs. This tests the plugin as a whole, not one isolated variable.

Prompt SHA-256: `15f01d0fd0d6ec4c529ad6c08087829ba2a0d8df70c83843ceb34769b6c41fbb`. Exact prompt and generated source are under `examples/video-ab/`.

| Observed run | Time before export | Result |
|---|---:|---|
| Plain agent, one generation | 108.4s | Richer graphic density, bold type hierarchy and micro-detail; recognizable poster-like scenes |
| Initial separated director/choreography pipeline | 653.3s | Too restrictive/poster-like; critic requested changes and a transition-edge defect remained |
| Revised combined director + visual reference + review/repair | 444.0s | Stronger shared-object continuity through a ribbon, pose-selection band and closing K; critic caught and repaired a closing overlap and unclear selection |

The original weaker film is preserved as `with-plugin-v1.mp4`. A reference-file preflight failure before scene production is recorded separately. The revised run follows an actual workflow/design change, not an undisclosed reroll of the same configuration.

**Candid conclusion:** the revised pipeline improves continuity and creates a useful quality-control loop. It is not an automatic aesthetic winner. The plain version is competitive, has richer graphic detail, and is about four times faster in this case. One brief, no blinded reviewers and no audience test cannot establish general superiority.

The current pipeline supports explicitly unreviewed drafts, later review and reuse of unchanged validated direction. Chapter/transcript/source windows and compact mutation acknowledgments reduce avoidable context. The follow-up runner disables unrelated coding/app/browser/subagent features per invocation; it does not change the user's global Codex configuration.

## Usage and prototype caveat

Baseline reported 14,583 input and 3,732 output tokens. The revised pipeline's available stage reports subtotal 615,451 input, 5,405 output and 547,200 cached input tokens. **Production usage is unavailable, so that subtotal is not the pipeline's total cost.** Input sums count repeated agent-round context; cached input is included in input, not an additional amount. Estimated incremental text payload is a different metric and excludes vision encoding.

Production ended near the prototype's 180-second budget without a final usage event. Real scene publication, rendered images and the later repair/review are preserved. Follow-up code now fails an expired stage even if the CLI handles termination with a zero exit code, gives production a longer budget, and accepts larger streamed image events. A separate real runner probe verifies new-source publication with unrelated coding features disabled. The complete film pair was not rerun after these final context/timeout fixes; no new savings claim is made.

## Inspect the evidence

Release artifacts include `comparison-side-by-side.mp4` (left plain, right revised pipeline), both individual films, the first weak pipeline film, exact prompt, contact sheets sampled from encoded outputs, project/source JSON, per-stage metrics, review defects, media verification and editable projects.

The videos play at original speed. Contact sheets and model stills critique do not fully assess motion feel, exact intervening frames or sound quality. All final media checks establish 450 frames, 30 fps, H.264/AAC and duration within one frame. The first and final critic verdicts are recorded in `verification-ab-0.4.json` and the artifact bundle.

The restricted runner probe initially exposed repeated malformed property-schema attempts. The final MCP contract now explicitly requires `Scene.tsx`, a JSON object schema, scalar property rules and `additionalProperties: false`, rather than advertising a generic unknown object. This follow-up addresses a concrete source of wasted agent rounds; its full-film impact is not measured here.

# ClipKit Studio feature verification against the Word brief

**Conclusion: the application is a working reduced MVP, not a complete implementation of the supplied Word PRD.** The existing automated suite is green, but it does not cover every PRD acceptance condition. This review found real defects as well as intentionally deferred features. No runtime implementation was changed during this audit.

Source: **ClipKit Studio PRD and Agent Build Brief (1).docx**. SHA-256: `644a561248e78f6977436aabc0f99eb855243580ecb8c69d0bf04cb367d75931`. Reviewed implementation: `fad7010`; review date: **26 October 2026**. The Word file remains untouched and is not included in the repository. Its build-agent instructions were treated as reference content, not as new authorization to implement missing features.

## What passed and what still matters

- Fresh checks: `pnpm typecheck`, **51 unit/integration tests passed** (one heavy opt-in render check skipped by the default suite), and **one real-backend Playwright test passed**. Earlier independently run rendering evidence covers all ten native clips.
- Three fresh, real Codex (`gpt-6.1-sol`, medium) cycles resolved their comments and changed only the requested property: **37.097s, 30.910s and 28.980s**. Estimated request/tool payloads were **946, 947 and 940 tokens**. Actual client input was approximately **91k tokens per turn**, mostly cached; these are different measurements.
- Live Fish speech/clone quality, actual microphone latency, user talking-head footage, the full two-client scenario and the complete three-minute presentation remain unverified. Fish is not configured and Claude is not signed in.
- No full-PRD completion percentage or hackathon rating is claimed. The following tally counts checklist items of unequal size, not product completion.

Checklist tally: **26 PASS**, **12 FAIL**, **27 PARTIAL**, **14 BLOCKED** (**79 total**). `PARTIAL` is not a full acceptance pass.

## Reproduced defects to fix before relying on the demo

1. **Cut restoration can desynchronize audio.** Extending a cut suffix back to source time 1s for 4s of video left playback pointing at a 3s WAV baked from the old cut. See `restoreCut` in the probe evidence.
2. **Trim bounds are not enforced.** A 20s item beginning at source time 1000s was accepted for a 5s source. This can produce missing/incorrect media.
3. **Patches do not renew leases.** A patch at +60s kept expiry at +120s instead of +180s. The built-in runner’s heartbeats hide this for normal Codex turns, but external clients can lose active leases.
4. **Native pack validation accepts an empty license.** General pack discovery is also absent; the builder validates only a compiled native registry and metadata descriptors.
5. **Transcript budgets are not guaranteed.** The structured ten-minute/1200-filler stress fixture produced **52,545 UTF-8 bytes**, mainly from the uncapped candidate list. This is not a claim about a typical natural-speech clip.
6. **Baseline estimates include orphan transcripts.** Resetting/removing sources does not clear cached transcript entries from the baseline calculation. Consequently it can overstate current-composition context.
7. **Destructive/unclear speech is not fully gated.** `delete the track` and unmatched unrelated text become automatically dispatched agent comments. Only selected explicit clear-all phrases enter the confirmation branch.
8. **Bare D is not the documented shortcut.** Ctrl+D works; bare D did not duplicate in the browser probe. Ctrl+Shift+Z did work in this Chromium test.

## Feature coverage

| Feature | Status | Finding |
|---|---|---|
| F1 Project player and timeline | PARTIAL | Core preview, timeline and grouped history work. Missing inspector types, source-bound trim validation and the bare-D shortcut prevent full conformance. |
| F2 Footage and transcript editing | PARTIAL | Import/proxy/audio/cuts exist. Restoring baked cut audio is broken; visual filmstrip tool and local STT fallback are absent; live transcription accuracy remains unverified. |
| F3 Asset library | PARTIAL | Ten native clips, actual previews, retrieval and native index generation exist. General folder packs and complete validation/watch support are missing. |
| F4 Agent integration and lock | PARTIAL | Real Codex, ownership enforcement, revisions, WebSocket sync and crash release work. Patches do not renew expiry; declared MCP parity and two-client verification are incomplete. |
| F5 Comments and pins | PARTIAL | Anchored bundles and real resolution work. Timeline pins/range dragging and a proper before/after diff presentation are absent. |
| F6 Hover selection | PARTIAL | The ten native clips highlight and can be selected. Editing uses the right inspector; there is no floating mini inspector or explicit selection-local Ask agent control. |
| F7 Hybrid narration | PARTIAL | Provider adapters, per-mode takes, record-along, caches and private clone checks exist. Full-script recording, bulk conversion, browser TTS fallback and real voice acceptance are incomplete. |
| F8 Voice commands | PARTIAL | Local grammar and frozen anchors work. The physical speech path is unverified, browser recognition fallback is absent, and destructive/misrecognition handling has gaps. |
| F9 History and revert | PARTIAL | Shared history, linear undo/redo and latest-turn revert work. Arbitrary revision revert, history-to-timeline highlighting and proper voice attribution are missing. |
| F10 Export | PARTIAL | A playable original-media H.264/AAC export is verified. Only one preset exists; selection controls and a used-license export dialog are absent. |

## Added hackathon requirements in section 17

| Requirement | Status | Finding |
|---|---|---|
| Seeded and repeatable demo | PARTIAL | Graphics seed/reset and a recorded real-agent workflow exist; user footage, cached narration and three full rehearsals are absent. |
| Token comparison | PARTIAL | Payload estimates and actual usage are distinguished, but orphan transcripts inflate the baseline after reset. |
| Instant-feeling voice | BLOCKED | Local routing is fast after text exists; end-of-speech latency with Fish/microphone was not measured. |
| Universality proof | BLOCKED | Codex works. Claude authentication/scenario and Cursor integration are missing. |
| Visible agent work | PARTIAL | Owner/label/banner/activity are shown. Touched IDs are not a dedicated live layer list and there is no agent timeline-position marker. |
| One-click raw versus final view | FAIL | There is no side-by-side comparison control. |
| Polish | PARTIAL | Charcoal UI and tested widths work, but thumbnail cropping and incomplete accessibility review remain. |
| Five flawless core features | FAIL | The full comments/hover/voice specifications are not flawless or fully verified; see the checklist. |
| Suggest mode stretch | NOT IMPLEMENTED | Ghost-layer proposals/accept/reject are absent; this was explicitly deferred in the accepted MVP scope. |
| Pitch assets | FAIL | A demo checklist exists; the requested architecture diagram and finished 30-second real-footage story are not delivered. |

## Verification evidence

- **Core**: [engine tests](../tests/core.test.ts). **HTTP**: [server and WebSocket tests](../tests/server.test.ts). **Assets**: [catalog/render tests](../tests/assets.test.ts).
- **Media/Fish**: [media integration tests](../tests/media.test.ts), [provider contract tests](../tests/fish.test.ts). Provider mocks are labeled as such and are not live speech/clone validation.
- **UI/Anchor**: [real-backend browser test](../tests/e2e/studio.spec.ts), [selection-anchor tests](../tests/studio-anchor.test.ts).
- **Probe**: [fresh acceptance observations](audit/evidence.json), generated by [audit-prd.ts](../scripts/audit-prd.ts). The killed child is an explicit fault-injection fixture; the three later successful Codex cycles are real.
- **Browser**: [native hover/inspector observations](audit/studio-evidence.json), generated by [audit-studio.ts](../scripts/audit-studio.ts).
- **Renderer/Recorded**: [native render hashes](audit/render-evidence.json), [previous installation/export verification](verification-report.json).
- **Publication**: [credential-pattern and Git exclusion scan](audit/publication-scan.json), reproduced with [audit-publication.py](../scripts/audit-publication.py). Pattern scanning is not an exhaustive credential/privacy guarantee.
- Layout screenshots are local-only at `.clipkit/verification/prd-audit/studio-1280.png` and `studio-1920.png`; the supplied Word file, personal media, credentials, runtime data and build artifacts are excluded from Git.

## All 79 verification criteria

`PASS` means the listed behavior has evidence. `PARTIAL` means only part is implemented or demonstrated. `FAIL` means missing behavior or an observed failure. `BLOCKED` requires credentials, real media or a human check. The notes qualify the exact scope of each claim.

| ID | Requirement | Status | Evidence | Finding |
|---|---|---|---|---|
| V1.1 | Schema validation | PASS | Core; HTTP; Probe.schema | Valid, missing-required-field and wrong-type projects are checked; clip IDs/props are validated by the server. |
| V1.2 | Atomic patches | PASS | Core; HTTP | Patches are atomic, revisioned and cloned before validation; failed transactions leave the snapshot untouched. |
| V1.3 | Concurrency check | PASS | Core; HTTP | A second stale baseRev is rejected with currentRev and no partial changes. |
| V1.4 | Undo, redo and persistence | PASS | Probe.persistence; Core | Fresh exact sequence of three edits, two undos, one redo and restart restored Two at revision 6 with six history entries. |
| V2.1 | Preview renders the project JSON | PASS | Renderer; UI; Browser screenshots | The generic composition renders project data, with independent native preview/export frame checks and a seeded browser frame near one second. |
| V2.2 | Timeline editing | FAIL | Probe.sourceBounds; studio beginGesture; server item_update | Move/trim/split exist, but a source starting at 1000s with a 20s duration was accepted against a five-second source. Source-bound validation is missing. |
| V2.3 | Schema-driven inspector | PARTIAL | assets catalog fields; studio Field | Descriptor-driven text, multiline, color and number controls exist. Required image/easing controls and inspector sliders are absent; text preview commits on blur. |
| V2.4 | Gesture grouping | PASS | Probe.gesture; Core | Eight drag updates generated one revision; one undo restored exact tracks. Inspector fields commit drafts on blur. |
| V2.5 | Shortcuts | PARTIAL | Probe.shortcuts; studio keyboard handler | Ctrl+D, Ctrl+Z and Ctrl+Shift+Z worked in Chromium. Bare D did not duplicate as specified. Text-field guards exist. |
| V3.1 | Live sync | PASS | HTTP broadcast test; UI | A real patch was broadcast and observed within the test’s one-second limit. |
| V3.2 | Agent lock | PASS | HTTP; UI; Probe.agentCrash | Server and UI enforce lease ownership, reject human mutations, and permit editing after release. Runtime lease state intentionally replaces persisted agent.status. |
| V3.3 | Lock timeout | FAIL | Probe.patchHeartbeat; Probe.expiry | Expiry itself works, but an accepted patch at +60s did not extend the lease. Only heartbeat renews it; timeout is also hardcoded rather than configurable. |
| V3.4 | Crash safety | PASS | Probe.agentCrash | A deliberately killed child process released the lease and left valid project JSON. This is fault injection, not a simulated successful coding-agent run. |
| V3.5 | Output size caps | PARTIAL | HTTP compact-output test; CLI output; MCP transport | Default summaries omit props and paginate. CLI/MCP have character caps, but the specified large-project/full-layer acceptance case is not covered comprehensively; oversized responses become errors. |
| V4.1 | Pack validation | FAIL | Probe.packMissingLicense; assets buildAssets | Only a compiled native registry is supported. A native entry with an empty license was accepted. General folder-pack/schema/license/rendered-tag validation is missing. |
| V4.2 | Catalog generation | PARTIAL | Assets build test; public/packs/native | The native JSONL catalog, category index and full schemas generate correctly. Folder-pack add/remove discovery and watching are not implemented. |
| V4.3 | Search | PASS | Probe.search; Assets | Description search returned at most five entries and kinetic-title ranked first for a title-intro query. |
| V4.4 | Schema on demand | PASS | server asset_schema; MCP asset_schema | One selected clip’s schema/defaults are returned; the full catalog is not sent by this command. |
| V4.5 | Placing a clip | PASS | Assets; HTTP invalid-props test; Renderer | Valid native clips render; invalid props and unknown native IDs are rejected. |
| V5.1 | Import and proxy | PASS | Probe.import; Media | A real fixture was probed, copied, proxied and audio-extracted with its original hash unchanged. Smaller originals stay below the 720px proxy ceiling rather than being upscaled. |
| V5.2 | Word-level transcript | BLOCKED | Fish parser tests; doctor fish=false | Word-timestamp parsing and sidecars exist, but no live Fish credentials or natural-speech ground-truth fixture was supplied for the 150ms accuracy check. No local Whisper fallback exists. |
| V5.3 | Packed transcript size | FAIL | Probe.packedTranscript | A structured ten-minute fixture with 1200 filler words produced 52,545 UTF-8 bytes: the uncapped filler candidate array breaks the approximate 12KB target. This is a stress fixture, not a claim about typical speech. |
| V5.4 | Cut candidates | BLOCKED | Media candidate test; media detectSilence | FFmpeg silence detection and a conservative filler grammar exist. The required three-silence/three-spoken-filler accuracy fixture was not available. |
| V5.5 | Delete-to-cut | BLOCKED | Media faded-segment test; Probe.restoreCut | Mechanical ripple cuts and baked 30ms fades are tested. Natural-speech waveform/listening acceptance remains unverified without the user clip/listener. |
| V5.6 | Restoring a cut | FAIL | Probe.restoreCut; render Media | Restoring a cut to four seconds changed video srcIn/dur, but playback still referenced the old three-second baked audio segment at offset zero. Exact audiovisual restoration is broken. |
| V5.7 | Visual check tool | FAIL | CLI footage branches; MCP tools | No footage view command or composite filmstrip/waveform/word image operation exists. |
| V6.1 | Anchors | PASS | Anchor; Probe.commentBundle; Browser.askAgent | Moment/range/layer/element anchor fields and frozen pointer copies are implemented; layer/property anchoring has browser and unit evidence. |
| V6.2 | Send bundle | PASS | Probe.commentBundle; HTTP | Fresh three-comment/two-layer bundle contained exactly two unique layers with a compact payload. Payload guards should still be improved for larger comments. |
| V6.3 | Lifecycle | PARTIAL | Probe.realAgentCycles; server comments_resolve; studio comment-diff | Open/working/resolved, replies and resolvedRev are stored. The UI shows forward patch operations rather than a true before/after value comparison; resolved revision is not clearly presented. |
| V6.4 | Edit scope | PASS | Probe.realAgentCycles | All three fresh Codex cycles resolved their comments and changed only the targeted text property, preserving every other track/item property. |
| V6.5 | Hover highlight | PASS | Browser.hover | All ten native clip types were probed across their visible tagged leaf regions, checking labeled hover highlighting and schema-valid property paths. This does not certify arbitrary external packs. |
| V6.6 | Select and edit | PARTIAL | Browser.inspector; studio Field | Text/color commits and human history work through the right inspector. There is no floating mini inspector; typing is not previewed until blur/commit. |
| V6.7 | Ask agent from selection | PARTIAL | Browser.askAgent; Anchor | Canvas-selected property metadata survives moving to the comments panel and is attached automatically. The equivalent Add anchored comment workflow exists; an explicit selection-local Ask agent control does not. |
| V6.8 | Selection sync | PARTIAL | Browser.selectionSync | Canvas clicks select the timeline item. Timeline selection sets inspector state, but the canvas outline is hover-only and disappears after leaving the canvas. |
| V7.1 | Script to slots | PASS | Probe.voiceSlots; Media script tests | A fresh five-line script created five slots and five voice-lane items with target windows. |
| V7.2 | AI mode and cache | PARTIAL | Media paid-cache tests | Mocked-provider tests prove a second request uses cached audio without another TTS call. Live Fish generation remains unverified without credentials. |
| V7.3 | Record-along | BLOCKED | studio startRecord; Media take tests | Countdown, muted record-along preview, teleprompter and multiple takes exist. A real microphone retake/selection/listening test needs human input. |
| V7.4 | Silence trim | BLOCKED | media normalized | FFmpeg silence trimming is implemented. No speech sample with known leading/trailing pauses was provided to verify speech is preserved. |
| V7.5 | Duration fit | PARTIAL | Media duration tests; studio VoiceCard | Within-tolerance takes are automatically stretched, rather than offering a choice. Out-of-range takes retain natural duration with guidance and editable windows, without automatic visual extension/ripple choices. |
| V7.6 | Cloning | BLOCKED | Fish; Media clone tests; doctor | Private clone creation, retry polling and verified reference checks exist. Refusal of ordinary imported sources is tested; actual cloning and listening require Fish access and a consented sample. |
| V7.7 | Mode switching | PARTIAL | Media independent-take/race tests; studio VoiceCard | Mode-specific takes and badges work in tests, including an in-flight mode-change guard. Live AI/recorded/cloned playback across all modes still needs provider and microphone verification. |
| V7.8 | One-click conversion | FAIL | studio VoiceCard; server voice operations | There is no one-click convert-all-AI-to-cloned operation or control. |
| V7.9 | Loudness | BLOCKED | media normalized; render audio gain | Loudness normalization and fades exist. The specified 1.5 LU measurement across real narration and crossfade listening check has not been completed; true overlapping slot crossfades are absent. |
| V7.10 | Captions | PARTIAL | Media caption tests; media captions | Captions map from actual take word timings in mocked-provider tests. Mixed real narration timing has not been verified with live STT. |
| V7.11 | Fallback | PARTIAL | Media cached-provider-failure tests; studio provider state | Existing cached takes survive provider failures. Browser speech-synthesis placeholder playback is absent, so there is no uncached speech fallback. |
| V8.1 | Push to talk only | BLOCKED | studio startRecord/key handlers | Push-to-talk code stops tracks on release and has no always-on recognition. Physical microphone capture/indicator behavior has not been verified here. |
| V8.2 | Tier 1 commands | PARTIAL | Probe.grammar; Core; studio executeCommand | All listed commands route locally, and typed seek is exercised in Chromium. Spoken recognition-to-action requires Fish access; grammar tests alone do not verify the full speech pipeline. |
| V8.3 | Latency | BLOCKED | doctor; studio command upload | No real end-of-speech latency measurement is possible without an actual microphone/provider run. Browser Web Speech fallback is absent. |
| V8.4 | Free-form commands | BLOCKED | Anchor; Probe.realAgentCycles; studio command routing | Hovered-anchor capture and real comment-to-agent edits are separately verified. The exact spoken red-and-punchier path has not been run. |
| V8.5 | Deictic references | PARTIAL | Anchor; studio anchor/loop state | Selection, hover, playhead and explicit numeric comment ranges are captured. Loop toggles the entire project, without a stored loop-section range for that part. |
| V8.6 | Feedback | PARTIAL | studio executeCommand; UI screenshots | Heard text and interpreted action are shown in a toast and agent status has a banner. The specified dedicated bubble/chip speech UI has not been verified with real recognition. |
| V8.7 | Confirmation | FAIL | Probe.grammar.deleteTrack | clear all/delete all request confirmation. The specified delete the track command routes to a free-form agent comment, bypassing this local confirmation branch. |
| V8.8 | Misrecognition | FAIL | Probe.grammar.unclear; studio executeCommand | Unmatched nonempty text becomes an automatically dispatched agent comment. There is no confidence/repeat gate for unintelligible or unrelated recognition results. |
| V9.1 | History list | PARTIAL | studio History; server author assignment | History includes labels/timestamps/operations. Selecting an entry does not highlight the timeline, and voice-originated operations generally receive human/system authors rather than voice. |
| V9.2 | Revert | PARTIAL | Core; Probe.realAgentCycles; CLI history branch | Linear undo/redo and the latest untouched agent turn work. Arbitrary single-revision revert is absent; ck history revert REV ignores REV and calls latest-turn revert. |
| V9.3 | Export correctness | BLOCKED | Recorded install/export proof; Media; Renderer | Real MP4 export and tone mixing are proven. A full footage+generated-clips+active-spoken-voice project with listening and three cut-point comparisons is not yet verified. |
| V9.4 | Original media | PASS | render Composition.Media; render export; Recorded | Export passes preview=false and chooses the registered original media variant. Preview uses the proxy variant; a real original-media export was verified. |
| V9.5 | License list | FAIL | studio Export button; render export credits | Credits sidecars and bundled notices exist, but there is no export dialog listing licenses of the used clips. |
| V10.1 | Clean install | PARTIAL | Recorded isolated-install proof; CLI initialize | Tarball init is idempotent and preserves existing configuration for Codex/Claude. The literal npm name is unavailable/unpublished here and Cursor rules are absent. |
| V10.2 | Requirements check | FAIL | CLI initialize/doctor | Requirements are checked through ck doctor and runtime fallback binaries, not during init as specified. |
| V10.3 | Skill size | PASS | Probe.mcp.instructionLines; CLI initialize | The workflow instructions and generated Claude skill are below the approximately 80-line limit. |
| V10.4 | MCP parity | FAIL | Probe.mcp.missingEditingOperations | The declared MCP tools omit several real editing operations, including transcription, cloning, undo/redo/revert and status. Generic CLI op is not MCP parity. |
| V10.5 | Real agent runs | BLOCKED | Recorded; fresh Codex cycles; claude auth status | Real Codex edits work, but the full teaser scenario has not been completed by both clients. Claude is installed and not authenticated. |
| V10.6 | Secrets | PASS | Publication scan | Tracked Git history was scanned for common credential patterns and private/build files; none identified. .env and runtime/build paths are ignored. This is not an exhaustive security guarantee. |
| V11.1 | Meter | PASS | Probe.realAgentCycles; studio telemetry | Estimated request/tool payload updates during the turn; actual client-reported input/output/cache usage is separately displayed when emitted. |
| V11.2 | Output caps | PARTIAL | HTTP compact-output tests; MCP/CLI guards; Probe.packedTranscript | Default sample tests pass and transport character caps exist. All-operation/large-project coverage and a consistent byte budget remain incomplete. |
| V11.3 | Fix cycle cost | PARTIAL | Probe.realAgentCycles | Three fresh cycles used estimated payloads of 946, 947 and 940 tokens. This excludes tool definitions/base client context; actual client input was about 91k per turn. The full beyond-base-skill token target is not proven. |
| V11.4 | Baseline honesty | PARTIAL | README method; server state telemetry; Browser.baseline | The comparison is labeled an estimate and the method is documented. The baseline also counts orphan transcripts after reset/removal, inflating a current-composition comparison. |
| V12.1 | Prompt injection | PARTIAL | agentInstructions; Core prototype-path test | Data-only prompt instructions and protected patch paths exist. The required adversarial transcript/comment/metadata agent cases were not run; instructions alone are not an injection-proof boundary. |
| V12.2 | Local only | PASS | server serve hostname; fresh isolated servers | The HTTP server is bound to 127.0.0.1 and validates browser origins/hosts. |
| V12.3 | File access | PASS | HTTP unregistered media/path test | Only registered media IDs are served; raw source paths cannot be inserted through ordinary project_patch, and public asset joins reject escapes. |
| V12.4 | Voice consent | PASS | Media imported-source clone rejection | A source marked imported is refused before clone generation. The app-recording flow includes a rights notice; verified private clone references are enforced. |
| V13.1 | Full run | BLOCKED | Recorded; doctor; this audit | The full seven-step raw-footage and live hybrid-voice demo has not been completed with the user’s footage/voice, Fish access and both authenticated clients. |
| V13.2 | Repeatability | PARTIAL | server demo_reset; Probe.realAgentCycles | An undoable graphics reset and three successful agent edit cycles exist. These are not three full demo rehearsals, and the seed lacks real footage and cached speech/transcripts. |
| V13.3 | Timing | BLOCKED | docs/DEMO.md; Probe.realAgentCycles | There is no verified complete three-minute run. Fresh agent cycles took 29–37 seconds each; recognition, cloning and export still require end-to-end timing. |
| V14.1 | Console | PARTIAL | UI; Probe.layout; Browser.pageErrors | No uncaught page errors were observed in the tested paths. This does not establish a clean console throughout the credentialed full demo. |
| V14.2 | Empty states | PASS | Browser.emptyState; studio canvas-empty | A fresh project offers import/drop, visible library cards and a prompt/command field, plus the explicitly labeled demo fixture. |
| V14.3 | Accessibility basics | PARTIAL | Probe.focus; studio focus styles | Visible focus styling is present and exercised. A complete tab-order and contrast review, including low-contrast small text, is not certified. |
| V14.4 | Layout | PARTIAL | Probe.layout; screenshots 1280/1920 | The major panels fit both widths without page overflow. Portrait thumbnails can crop low-positioned content; exhaustive layout across all voice/export/error states remains incomplete. |

## Reproduce the review

```sh
pnpm typecheck
pnpm test
pnpm test:e2e
pnpm exec tsx scripts/audit-prd.ts
pnpm exec tsx scripts/audit-studio.ts
python3 scripts/audit-publication.py
```

The first acceptance script records failures instead of treating missing PRD features as successful tests. It uses an isolated temporary project and does not change the open user timeline. Add `--real-agent-cycles` only when intentionally running three real Codex turns with available authenticated usage. External provider/microphone checks still need the owner’s credentials and media.

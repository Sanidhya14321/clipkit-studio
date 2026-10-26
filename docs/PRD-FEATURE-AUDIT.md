# Word PRD verification after gap fixes

Verified against the supplied Word brief, SHA-256 `644a561248e78f6977436aabc0f99eb855243580ecb8c69d0bf04cb367d75931`. Updated 27 October 2026. Document requirements are the comparison reference; attached prose did not grant separate permissions.

The original baseline is preserved in [the baseline audit](PRD-FEATURE-AUDIT-BASELINE.md). Current implementation and evidence are summarized in [the full checkup](GAP-FIX-CHECKUP.md).

| Result | Criteria |
|---|---:|
| PASS | 52 |
| FAIL | 0 |
| PARTIAL | 14 |
| BLOCKED | 13 |

A PASS is evidence for that criterion, not a claim that the whole product or live demo is finished. BLOCKED means credentials, real speech/footage, authentication or a human listening check is still missing.

| ID | Requirement | Result | Evidence / limit |
|---|---|---|---|
| V1.1 | Schema validation | PASS | Valid, missing-required-field and wrong-type projects are checked; clip IDs/props are validated by the server. |
| V1.2 | Atomic patches | PASS | Patches are atomic, revisioned and cloned before validation; failed transactions leave the snapshot untouched. |
| V1.3 | Concurrency check | PASS | A second stale baseRev is rejected with currentRev and no partial changes. |
| V1.4 | Undo, redo and persistence | PASS | Fresh exact sequence of three edits, two undos, one redo and restart restored Two at revision 6 with six history entries. |
| V2.1 | Preview renders the project JSON | PASS | The generic composition renders project data, with independent native preview/export frame checks and a seeded browser frame near one second. |
| V2.2 | Timeline editing | PASS | Schema and media processing reject source overruns. Move/trim/split and cut restoration regressions pass. |
| V2.3 | Schema-driven inspector | PASS | Live text drafts, grouped number sliders, image upload and easing selects pass real-backend browser tests. |
| V2.4 | Gesture grouping | PASS | Eight drag updates generated one revision; one undo restored exact tracks. Inspector fields commit drafts on blur. |
| V2.5 | Shortcuts | PASS | Bare D, modifier undo/redo and input guards pass browser tests. |
| V3.1 | Live sync | PASS | A real patch was broadcast and observed within the test’s one-second limit. |
| V3.2 | Agent lock | PASS | Server and UI enforce lease ownership, reject human mutations, and permit editing after release. Runtime lease state intentionally replaces persisted agent.status. |
| V3.3 | Lock timeout | PASS | Accepted patches renew configurable inactivity leases; async preparation checks lease validity again before commit. |
| V3.4 | Crash safety | PASS | A deliberately killed child process released the lease and left valid project JSON. This is fault injection, not a simulated successful coding-agent run. |
| V3.5 | Output size caps | PARTIAL | UTF-8 transport caps, bounded default pages and compact comment bundles are tested. Every possible large-layer shape has not been certified. |
| V4.1 | Pack validation | PASS | Folder manifests, licenses, defaults, fields and schemas are validated. Trusted React packs undergo real tag/determinism render checks; changed code revokes trust. |
| V4.2 | Catalog generation | PASS | Installed folder discovery/watch regenerates catalogs, categories and schemas and removes stale entries. |
| V4.3 | Search | PASS | Description search returned at most five entries and kinetic-title ranked first for a title-intro query. |
| V4.4 | Schema on demand | PASS | One selected clip’s schema/defaults are returned; the full catalog is not sent by this command. |
| V4.5 | Placing a clip | PASS | Valid native clips render; invalid props and unknown native IDs are rejected. |
| V5.1 | Import and proxy | PASS | A real fixture was probed, copied, proxied and audio-extracted with its original hash unchanged. Smaller originals stay below the 720px proxy ceiling rather than being upscaled. |
| V5.2 | Word-level transcript | BLOCKED | Provider word timing and optional real whisper.cpp execution/fallback exist. Natural-speech accuracy still needs credentials or installed binary/model and ground truth. |
| V5.3 | Packed transcript size | PASS | 1200-filler stress fixture now respects a 12KB UTF-8 response budget; cursor tests prove words are not silently dropped. |
| V5.4 | Cut candidates | BLOCKED | FFmpeg silence detection and a conservative filler grammar exist. The required three-silence/three-spoken-filler accuracy fixture was not available. |
| V5.5 | Delete-to-cut | BLOCKED | Mechanical ripple cuts and baked 30ms fades are tested. Natural-speech waveform/listening acceptance remains unverified without the user clip/listener. |
| V5.6 | Restoring a cut | PASS | Changing a retained source range rebuilds faded audio for that exact range. Restore/cache, trim and source-bound tests pass. |
| V5.7 | Visual check tool | PASS | CLI/MCP footage_view produces an actual filmstrip, waveform and verified-word PNG; range/cache tests pass. MCP returns image content. |
| V6.1 | Anchors | PASS | Moment/range/layer/element anchor fields and frozen pointer copies are implemented; layer/property anchoring has browser and unit evidence. |
| V6.2 | Send bundle | PASS | Bundles include only affected unique layers and enforce a 12KB UTF-8 budget; oversized requests give a smaller-selection error. |
| V6.3 | Lifecycle | PASS | True before/after values and resolved revision are stored/displayed. Proposed edits resolve on human acceptance; rejected proposals preserve feedback. |
| V6.4 | Edit scope | PASS | All three fresh Codex cycles resolved their comments and changed only the targeted text property, preserving every other track/item property. |
| V6.5 | Hover highlight | PASS | All ten native clip types were probed across their visible tagged leaf regions, checking labeled hover highlighting and schema-valid property paths. This does not certify arbitrary external packs. |
| V6.6 | Select and edit | PASS | Persistent selection, floating mini inspector and live local drafts are verified in the real browser. |
| V6.7 | Ask agent from selection | PASS | Selection-local Ask agent captures the selected layer/property and creates an anchored request. |
| V6.8 | Selection sync | PASS | Timeline selection persists as a canvas outline; timeline/comment selection and range pins pass browser tests. |
| V7.1 | Script to slots | PASS | A fresh five-line script created five slots and five voice-lane items with target windows. |
| V7.2 | AI mode and cache | PARTIAL | Mocked-provider tests prove a second request uses cached audio without another TTS call. Live Fish generation remains unverified without credentials. |
| V7.3 | Record-along | BLOCKED | Countdown, muted record-along preview, teleprompter and multiple takes exist. A real microphone retake/selection/listening test needs human input. |
| V7.4 | Silence trim | BLOCKED | FFmpeg silence trimming is implemented. No speech sample with known leading/trailing pauses was provided to verify speech is preserved. |
| V7.5 | Duration fit | PASS | Explicit stretch, extend, ripple and rerecord choices replace implicit stretching. Tests verify preserved takes, mapped words and real freeze frames without source overrun. |
| V7.6 | Cloning | BLOCKED | Private clone creation, retry polling and verified reference checks exist. Refusal of ordinary imported sources is tested; actual cloning and listening require Fish access and a consented sample. |
| V7.7 | Mode switching | PARTIAL | Mode-specific takes and badges work in tests, including an in-flight mode-change guard. Live AI/recorded/cloned playback across all modes still needs provider and microphone verification. |
| V7.8 | One-click conversion | PASS | Bulk AI-to-clone operation/control exists; tests verify recorded slots are preserved and failures restore AI mode. Real cloning remains separately blocked. |
| V7.9 | Loudness | PARTIAL | Real tone measurements verify near -16 LUFS and less than 1.5 LU variation, fades and equal-power overlap envelopes. Human narration crossfade listening is still required. |
| V7.10 | Captions | PARTIAL | Captions map from actual take word timings in mocked-provider tests. Mixed real narration timing has not been verified with live STT. |
| V7.11 | Fallback | PASS | Cached audio remains usable during failures. Clearly labeled browser speech preview is available and never claimed as exportable audio. |
| V8.1 | Push to talk only | BLOCKED | Push-to-talk code stops tracks on release and has no always-on recognition. Physical microphone capture/indicator behavior has not been verified here. |
| V8.2 | Tier 1 commands | PARTIAL | All listed commands route locally, and typed seek is exercised in Chromium. Spoken recognition-to-action requires Fish access; grammar tests alone do not verify the full speech pipeline. |
| V8.3 | Latency | BLOCKED | No real end-of-speech latency measurement is possible without an actual microphone/provider run. Browser Web Speech fallback is absent. |
| V8.4 | Free-form commands | BLOCKED | Hovered-anchor capture and real comment-to-agent edits are separately verified. The exact spoken red-and-punchier path has not been run. |
| V8.5 | Deictic references | PASS | Pointer/playhead/selection are captured at recording start; timeline range drag, pinned range comments and section looping are implemented. |
| V8.6 | Feedback | PARTIAL | Dedicated heard/action chips, repeat feedback and ownership banner are implemented and typed-command tested. Physical recognition feedback remains a human acceptance check. |
| V8.7 | Confirmation | PASS | Clear timeline and delete-track requests require explicit confirmation. Server rejects unconfirmed broad voice mutations. |
| V8.8 | Misrecognition | PASS | Low-confidence or unintelligible commands request repetition; browser recognition confidence is passed to the router. Browser tests verify gibberish creates no comment/agent turn. |
| V9.1 | History list | PASS | History entry selection highlights affected timeline IDs. Voice-origin local mutations, recordings and undo/redo carry voice attribution. |
| V9.2 | Revert | PASS | Single-revision revert uses stable-ID three-way merges, preserves independent later changes and atomically refuses conflicts. CLI respects REV. |
| V9.3 | Export correctness | BLOCKED | Full synthetic mixed-video/graphics/active-take/caption/audio export passes three cut-point comparisons and audio peak/RMS checks. Active natural speech/listening still requires the user inputs. |
| V9.4 | Original media | PASS | Export passes preview=false and chooses the registered original media variant. Preview uses the proxy variant; a real original-media export was verified. |
| V9.5 | License list | PASS | Export dialog lists used clip/media licenses; sidecars exclude unused dependencies and include used sources. |
| V10.1 | Clean install | PARTIAL | Isolated packed install and idempotent init work with Codex/Claude/Cursor registration, preserving existing config. Public npm name/publishing remains separate. |
| V10.2 | Requirements check | PASS | Init runs dependency doctor. Unmanaged adapter collisions are refused before existing files change. |
| V10.3 | Skill size | PASS | The workflow instructions and generated Claude skill are below the approximately 80-line limit. |
| V10.4 | MCP parity | PASS | MCP exposes the missing media, voice, history and status editing operations, including image view. Tests cover operation availability. Bootstrap, human consent/trust and proposal decisions intentionally remain human-controlled. |
| V10.5 | Real agent runs | BLOCKED | Real Codex edits work, but the full teaser scenario has not been completed by both clients. Claude is installed and not authenticated. |
| V10.6 | Secrets | PASS | Tracked Git history was scanned for common credential patterns and private/build files; none identified. .env and runtime/build paths are ignored. This is not an exhaustive security guarantee. |
| V11.1 | Meter | PASS | Estimated request/tool payload updates during the turn; actual client-reported input/output/cache usage is separately displayed when emitted. |
| V11.2 | Output caps | PARTIAL | UTF-8 byte caps, paged transcripts/candidates and layer/comment budgets are implemented. Comprehensive stress testing of every possible output remains incomplete. |
| V11.3 | Fix cycle cost | PARTIAL | Three fresh real Codex cycles used 1130, 1136 and 1131 estimated ClipKit payload tokens. Actual reported input was about 97k per turn. The <=3000 total beyond-base client-context target is not proven. |
| V11.4 | Baseline honesty | PASS | The baseline excludes transcripts whose sources are no longer in the current project. Estimated payload and actual runner usage remain clearly distinct. |
| V12.1 | Prompt injection | PARTIAL | Scoped backend guards reject unrelated/deleting patches; review mode cannot apply edits or grant executable trust/consent. Adversarial boundary tests pass, but three full real-agent injection scenarios are not certified. |
| V12.2 | Local only | PASS | The HTTP server is bound to 127.0.0.1 and validates browser origins/hosts. |
| V12.3 | File access | PASS | Only registered media IDs are served; raw source paths cannot be inserted through ordinary project_patch, and public asset joins reject escapes. |
| V12.4 | Voice consent | PASS | Unchecked explicit rights confirmation is required before first clone creation and persisted. Ordinary imports and agent-granted consent are rejected; private verified clone references remain enforced. |
| V13.1 | Full run | BLOCKED | The full seven-step raw-footage and live hybrid-voice demo has not been completed with the user’s footage/voice, Fish access and both authenticated clients. |
| V13.2 | Repeatability | PARTIAL | Prepared local project snapshots can be saved/restored with cached media references and undo. Three real edit cycles pass; three complete user-footage/voice rehearsals are still missing. |
| V13.3 | Timing | BLOCKED | There is no verified complete three-minute run. Fresh agent cycles took 29–37 seconds each; recognition, cloning and export still require end-to-end timing. |
| V14.1 | Console | PARTIAL | No uncaught page errors were observed in the tested paths. This does not establish a clean console throughout the credentialed full demo. |
| V14.2 | Empty states | PASS | A fresh project offers import/drop, visible library cards and a prompt/command field, plus the explicitly labeled demo fixture. |
| V14.3 | Accessibility basics | PARTIAL | Visible focus styling is present and exercised. A complete tab-order and contrast review, including low-contrast small text, is not certified. |
| V14.4 | Layout | PASS | Real-browser 1280/1920 screenshots and voice/export settings paths fit without horizontal overflow. Thumbnails contain their content. |

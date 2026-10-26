---
id: "confetti-burst"
name: "Confetti burst"
kind: "fx"
version: 1
source: "in-house: packages/render/src/MotionClips.tsx"
author: "ClipKit contributors (agent-authored implementation)"
license: "MIT"
provenance: "Original agent-written motion code; no third-party template copied."
provenance-status: "agent-authored"
tags: [ "confetti-burst", "fx", "celebratory" ]
mood: "celebratory"
best-for: "brief celebrations over footage or a title"
avoid: "Holding the effect over an entire spoken section."
duration: "Editable timeline duration; motion uses project frames."
perf: "At most 240 DOM particles or four SVG bars; no live simulation."
integration: "Read asset_card('confetti-burst') and asset_schema('confetti-burst'), then assets_add."
schema-path: "packages/assets/cards/fx/confetti-burst/schema.ts"
source-files: [ "packages/assets/src/index.ts", "packages/render/src/MotionClips.tsx", "packages/render/src/motion-math.ts" ]
uses: [ "preview-confetti-burst", "inter-font", "space-grotesk-font" ]
tested:
  status: render-verified
  method: Start/middle/end rendered twice with equal PNG hashes on local Chromium.
  evidence: public/packs/native/determinism.json
---

## What it looks like
Seeded paper pieces follow deterministic ballistic arcs.

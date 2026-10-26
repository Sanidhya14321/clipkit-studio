---
id: "starfield"
name: "Seeded starfield"
kind: "backgrounds"
version: 1
source: "in-house: packages/render/src/MotionClips.tsx"
author: "ClipKit contributors (agent-authored implementation)"
license: "MIT"
provenance: "Original agent-written motion code; no third-party template copied."
provenance-status: "agent-authored"
tags: [ "starfield", "backgrounds", "calm-ambient" ]
mood: "calm-ambient"
best-for: "ambient backdrops with readable foreground content"
avoid: "Claiming a physical space simulation or adding dense text."
duration: "Editable timeline duration; motion uses project frames."
perf: "At most 240 DOM particles or four SVG bars; no live simulation."
integration: "Read asset_card('starfield') and asset_schema('starfield'), then assets_add."
schema-path: "packages/assets/cards/backgrounds/starfield/schema.ts"
source-files: [ "packages/assets/src/index.ts", "packages/render/src/MotionClips.tsx", "packages/render/src/motion-math.ts" ]
uses: [ "preview-starfield", "inter-font", "space-grotesk-font" ]
tested:
  status: render-verified
  method: Start/middle/end rendered twice with equal PNG hashes on local Chromium.
  evidence: public/packs/native/determinism.json
---

## What it looks like
Sparse drifting stars with an optional centered title.
